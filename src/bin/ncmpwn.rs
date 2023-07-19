use clap::Parser;
use do_notation::m;
use ncmpwn::{ncmdump::NcmDump, qmcdump::QmcDump};
use thiserror::Error;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
use std::io;
use std::iter::Iterator;
use std::path;
use std::sync::mpsc;
use std::thread;
use std::env;

#[derive(Debug, Parser)]
struct CliArgs {
    #[arg(short, long)]
    pub ncm: Vec<path::PathBuf>,
    #[arg(short, long)]
    pub qmc: Vec<path::PathBuf>,

    /// Number of workers
    #[arg(short, long, default_value_t = 1)]
    pub worker: u8,

    /// Add tag for ncm files
    #[arg(short, long, default_value_t = true)]
    pub tag: bool,

    /// Output dir (default: PWD)
    #[arg(short, long)]
    pub output: Option<path::PathBuf>,
}

macro_rules! send_job {
    ($recvs:expr, $jobs:expr, $wrapper:expr) => {
        send!($recvs, $jobs.into_iter().map($wrapper))
    };
}

macro_rules! send {
    ($recvs:expr, $msg:expr) => {
        for (job, r) in $msg.into_iter().zip($recvs.into_iter().cycle()) {
            r.send(job).unwrap();
        }
    };
}

#[derive(Debug, Clone)]
enum Job {
    Ncm(path::PathBuf),
    Qmc(path::PathBuf),
    End,
}

fn main() {
    let args = CliArgs::parse();

    let mut txs = vec![];
    let mut handles = vec![];
    let output_dir = args.output.unwrap_or_else(|| env::current_dir().expect("Cannot get PWD"));
    for _ in 0..args.worker {
        let (tx, rx) = mpsc::channel();
        txs.push(tx);
        let output_dir = output_dir.clone();

        let handle = thread::spawn(move || loop {
            match rx.recv().unwrap() {
                Job::End => {
                    break;
                }
                Job::Ncm(fp) => {
                    ncmdump(&fp, &output_dir);
                }
                Job::Qmc(fp) => {
                    qmcdump(&fp, &output_dir);
                }
            }
        });
        handles.push(handle);
    }

    send_job!(txs.clone(), args.ncm, Job::Ncm);
    send_job!(txs.clone(), args.qmc, Job::Qmc);
    send!(
        txs.clone(),
        std::iter::repeat(Job::End).take(args.worker as usize)
    );

    for handle in handles {
        handle.join().unwrap();
    }
}

fn ncmdump(input: &path::Path, output_dir: &path::Path) {
    let res: Result<(), CliError> = m! {
        basename <- input.file_stem().ok_or(CliError::BaseNameError).map(|s| s.to_owned());
        basename <- basename.to_str().ok_or(CliError::BaseNameError);
        reader <- std::fs::File::open(input).map_err(|_| CliError::OpenError(input.to_owned()));
        dump <- NcmDump::from_reader(reader).map_err(|e| CliError::Other(e.to_string()));
        let mut dump = dump;
        info <- dump.get_info().map_err(|e| CliError::Other(e.to_string()));
        ext <- match ncmpwn::ncmdump::MediaFormat::from(info.format.as_str()) {
            ncmpwn::ncmdump::MediaFormat::fLaC => Ok("flac"),
            ncmpwn::ncmdump::MediaFormat::ID3v2 => Ok("mp3"),
            ncmpwn::ncmdump::MediaFormat::Unsupported => Err(CliError::UnsupportedFormat),
        };
        let output_file = format!("{basename}.{ext}");
        let mut output_dir = output_dir.to_owned();
        let _ = output_dir.push(output_file);
        write <- std::fs::File::options()
            .create(true)
            .write(true)
            .open(&output_dir)
            .map_err(|_| CliError::WriteError(output_dir.clone()));
        let mut write = write;

        dump.write_with_tag(&mut write).map_err(|_| CliError::WriteError(output_dir))
    };

    if let Err(e) = res {
        error!("{:?}: {}", input, e.to_string());
    }
}

fn qmcdump(input: &path::Path, output_dir: &path::Path) {
    let res: Result<(), CliError> = m! {
        basename <- input.file_stem().ok_or(CliError::BaseNameError).map(|s| s.to_owned());
        basename <- basename.to_str().ok_or(CliError::BaseNameError);
        ext <- input.extension().ok_or(CliError::NoFormat);
        ext <- ext.to_str().ok_or(CliError::NoFormat);
        ext <- match ext {
            "qmc3" => Ok("mp3"),
            "qmcflac" => Ok("flac"),
            _ => Err(CliError::UnsupportedFormat),
        };
        let output_file = format!("{basename}.{ext}");
        reader <- std::fs::File::open(input).map_err(|_| CliError::OpenError(input.to_owned()));
        let mut dump = QmcDump::from_reader(reader);
        let mut output_dir = output_dir.to_owned();
        let _ = output_dir.push(output_file);
        write <- std::fs::File::options()
            .create(true)
            .write(true)
            .open(&output_dir)
            .map_err(|_| CliError::WriteError(output_dir.clone()));
        let mut write = write;
        io::copy(&mut dump, &mut write)
            .map_err(|_| CliError::WriteError(output_dir.clone()))
            .map(|_| ())
    };

    if let Err(e) = res {
        error!("{:?}: {}", input, e.to_string());
    }
}

#[derive(Debug, Error)]
enum CliError {
    #[error("Cannot find out file basename")]
    BaseNameError,
    #[error("Cannot open source file: {0}")]
    OpenError(path::PathBuf),
    #[error("Cannot open target file: {0}")]
    WriteError(path::PathBuf),
    #[error("Unsupported format")]
    UnsupportedFormat,
    #[error("Cannot decide file format")]
    NoFormat,
    #[error("{0}")]
    Other(String),
}
