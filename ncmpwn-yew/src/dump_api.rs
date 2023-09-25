use std::{path::Path, io::{Cursor, Read}};
pub use ncmpwn::ncmdump::MediaFormat;
use ncmpwn::{ncmdump::{NcmInfo, NcmDump}, qmcdump::QmcDump};
pub use ncmpwn::ncmdump::{ error::DumpResult, error::Error as DumpError };
use image::ImageFormat;

pub fn guess_from_qmc_ext(path: &Path) -> MediaFormat {
    match path.extension() {
        None => MediaFormat::Unsupported,
        Some(ext) => match ext.to_str() {
            None => MediaFormat::Unsupported,
            Some(s) => match s {
                "qmcflac" => MediaFormat::fLaC,
                "qmc3" => MediaFormat::ID3v2,
                _ => MediaFormat::Unsupported,
            }
        }
    }
}

pub fn guess_from_ncm_info(info: &NcmInfo) -> MediaFormat {
    info.format.as_str().into()
}

pub fn decrypt_qmc(source: &[u8]) -> DumpResult<(MediaFormat, Vec<u8>)> {
    let reader = Cursor::new(source);
    let mut reader = QmcDump::from_reader(reader);

    let mut res = vec![];
    match reader.read_to_end(&mut res) {
        Ok(_) => Ok((reader.get_format(), res)),
        Err(e) => Err(e.into()),
    }
}

pub fn decrypt_ncm(source: &[u8]) -> DumpResult<(NcmInfo, Vec<u8>, Vec<u8>)> {
    let reader = Cursor::new(source);
    let mut reader = NcmDump::from_reader(reader)?;

    let info = reader.get_info()?;
    let image = reader.get_image()?;

    reader.move_to_start()?;
    let buf = vec![];
    // reader.read_to_end(&mut buf)?;
    let mut cursor = Cursor::new(buf);
    reader.write_with_tag(&mut cursor)?;

    Ok((info, image, cursor.into_inner()))
}

pub fn media_mime_to_ext(format: MediaFormat) -> &'static str {
    match format {
        MediaFormat::fLaC => ".flac",
        MediaFormat::ID3v2 => ".mp3",
        _ => "",
    }
}

#[allow(unused)]
pub fn img_to_element(format: ImageFormat, data: &str) -> String {
    match format {
        ImageFormat::Png => format!("data:image/png;base64,{}", data),
        ImageFormat::Jpeg => format!("data:image/jpeg;base64,{}", data),
        ImageFormat::Gif => format!("data:image/gif;base64,{}", data),
        ImageFormat::WebP => format!("data:image/webp;base64,{}", data),
        ImageFormat::Pnm => format!("data:image/pnm;base64,{}", data),
        ImageFormat::Tiff => format!("data:image/tiff;base64,{}", data),
        ImageFormat::Tga => format!("data:image/tga;base64,{}", data),
        ImageFormat::Dds => format!("data:image/dds;base64,{}", data),
        ImageFormat::Bmp => format!("data:image/bmp;base64,{}", data),
        ImageFormat::Ico => format!("data:image/ico;base64,{}", data),
        ImageFormat::Hdr => format!("data:image/hdr;base64,{}", data),
        ImageFormat::OpenExr => format!("data:image/openexr;base64,{}", data),
        ImageFormat::Farbfeld => format!("data:image/farbeld;base64,{}", data),
        ImageFormat::Avif => format!("data:image/avif;base64,{}", data),
        ImageFormat::Qoi => format!("data:image/qoi;base64,{}", data),
        _ => data.to_string(),
    }
}

pub fn data_to_element(format: MediaFormat, data: &str) -> String {
    match format {
        MediaFormat::fLaC => format!("data:audio/flac;base64,{}", data),
        MediaFormat::ID3v2 => format!("data:audio/mp3;base64,{}", data),
        _ => "".to_owned(),
    }
}
