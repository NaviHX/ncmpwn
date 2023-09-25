use std::collections::HashMap;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use do_notation::m;
use gloo_file::callbacks::FileReader;
use gloo_file::{File, ObjectUrl};
use image::ImageFormat;
use ncmpwn::error::{DumpResult, Error as DumpError};
use ncmpwn::MediaFormat;
use ncmpwn::NcmInfo;
use uuid::Uuid;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use material_yew::top_app_bar_fixed::MatTopAppBarTitle;
use material_yew::MatCircularProgress;
use material_yew::MatTopAppBarFixed;

use crate::dump_api::{self, data_to_object_url, media_mime_to_ext};

// const WIDTH: u32 = 128;
// const HEIGHT: u32 = 128;

pub struct App {
    decryted_files: Vec<DecryptTask>,
    uuid_table: HashMap<Uuid, usize>,
    task_table: HashMap<Uuid, FileReader>,
}

enum DecryptTask {
    Decrypting(String),
    Finished(Box<DecryptedFile>),
    Error(String, String),
}

#[allow(unused)]
pub struct DecryptedFile {
    original_name: String,
    name: String,
    image: Option<(ImageFormat, String)>,
    object_url: ObjectUrl,
    info: Option<NcmInfo>,
}

impl DecryptedFile {
    pub fn new(
        original_name: &str,
        name: &str,
        image: Option<(ImageFormat, String)>,
        data: &[u8],
        info: Option<NcmInfo>,
    ) -> Self {
        // let base64_data = STANDARD.encode(data);
        let object_url = data_to_object_url(data);

        Self {
            original_name: original_name.to_string(),
            name: name.to_string(),
            image,
            // base64_data: (media_format, base64_data),
            object_url,
            info,
        }
    }
}

pub enum Msg {
    Add(Vec<File>),
    Finish(Uuid, Box<DecryptedFile>),
    Error(Uuid, String),
}

impl Component for App {
    type Message = Msg;

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            decryted_files: vec![],
            uuid_table: HashMap::new(),
            task_table: HashMap::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Add(files) => {
                for file in files {
                    let id = self.decryted_files.len();
                    let uuid = Uuid::new_v4();
                    let filename = file.name();
                    let link = ctx.link().clone();

                    self.decryted_files
                        .push(DecryptTask::Decrypting(filename.clone()));
                    self.uuid_table.insert(uuid, id);

                    let filepath: std::path::PathBuf = filename.clone().into();
                    let task_table = &mut self.task_table;
                    let qmc_format = dump_api::guess_from_qmc_ext(&filepath);
                    let res: DumpResult<()> = m! {
                        ext <- filepath.extension().ok_or(DumpError::FormatError);
                        ext <- ext.to_str().ok_or(DumpError::FormatError);
                        {
                            match ext {

                                // ===========
                                // === NCM ===
                                // ===========
                                "ncm" => {
                                    let task = gloo_file::callbacks::read_as_bytes(&file, move |res| {
                                        let res = m! {
                                            buf <- res.map_err(|e| DumpError::IO(e.to_string()));
                                            res <- dump_api::decrypt_ncm(&buf);
                                            let (info, image, data) = res;
                                            format <- match dump_api::guess_from_ncm_info(&info) {
                                                MediaFormat::Unknown | MediaFormat::Unsupported => Err(DumpError::FormatError),
                                                f => Ok(f),
                                            };
                                            image_format <- image::guess_format(&image).map_err(|_| DumpError::ImageFormatError);
                                            return (info, image, data, format, image_format);
                                        };

                                        match res {
                                            Ok((info, image, data, format, image_format)) => {
                                                // let image = {
                                                //     let image = ImageReader::new(Cursor::new(image)).with_guessed_format().unwrap().decode().unwrap();
                                                //     let image = image.resize(WIDTH, HEIGHT, image::imageops::FilterType::Triangle);
                                                //     image.into_bytes()
                                                // };
                                                let image = STANDARD.encode(image);
                                                let orig = filename.clone();
                                                let mut filename = info.name.clone();
                                                filename.push_str(media_mime_to_ext(format));
                                                let decrypted_file = Box::new(DecryptedFile::new(&orig, &filename, Some((image_format, image)), &data, Some(info)));
                                                link.send_message(Msg::Finish(uuid, decrypted_file));
                                            }
                                            Err(e) => {
                                                link.send_message(Msg::Error(uuid, e.to_string()));
                                            }
                                        }
                                    });

                                    task_table.insert(uuid, task);
                                    Ok(())
                                }

                                // ===========
                                // === QMC ===
                                // ===========
                                "qmc3" | "qmcflac" => {
                                    let task = gloo_file::callbacks::read_as_bytes(&file, move |res| {
                                        let res = m! {
                                            buf <- res.map_err(|e| DumpError::IO(e.to_string()));
                                            res <- dump_api::decrypt_qmc(&buf);
                                            let (_, data) = res;
                                            return (data, qmc_format);
                                        };

                                        match res {
                                            Ok((data, format)) => {
                                                let orig = filename.clone();
                                                let mut filename = filename;
                                                filename.push_str(media_mime_to_ext(format));
                                                let decrypted_file = Box::new(DecryptedFile::new(&orig, &filename, None, &data, None));
                                                link.send_message(Msg::Finish(uuid, decrypted_file))
                                            }
                                            Err(e) => {
                                                link.send_message(Msg::Error(uuid, e.to_string()));
                                            }
                                        }
                                    });

                                    task_table.insert(uuid, task);
                                    Ok(())
                                }
                                _ => Err(DumpError::FormatError),
                            }
                        }
                    };

                    match res {
                        Ok(_) => {}
                        Err(e) => ctx.link().send_message(Msg::Error(uuid, e.to_string())),
                    }
                }
            }
            Msg::Finish(uuid, file) => {
                if let Some(f) = self
                    .uuid_table
                    .get(&uuid)
                    .and_then(|&id| self.decryted_files.get_mut(id))
                {
                    *f = DecryptTask::Finished(file);
                } else {
                    log::error!("Cannot found {} task", uuid);
                }
            }
            Msg::Error(uuid, e) => {
                log::error!("{}", e);

                if let Some(f) = self
                    .uuid_table
                    .get(&uuid)
                    .and_then(|&id| self.decryted_files.get_mut(id))
                {
                    let n = match f {
                        DecryptTask::Decrypting(s) => s.clone(),
                        _ => "ERROR".to_owned(),
                    };

                    *f = DecryptTask::Error(n, e);
                }
            }
        };

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_submit = ctx.link().callback(move |e: Event| {
            let mut submitted_files = vec![];
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                let files = js_sys::try_iter(&files)
                    .unwrap()
                    .unwrap()
                    .map(|v| web_sys::File::from(v.unwrap()))
                    .map(File::from);
                submitted_files.extend(files);
            }

            Msg::Add(submitted_files)
        });

        html! {
            <div>
                <MatTopAppBarFixed>
                    <MatTopAppBarTitle>
                        <div class="app-title">
                            <h1>{"Ncmpwn Wasm"}</h1>
                        </div>
                    </MatTopAppBarTitle>
                </MatTopAppBarFixed>
                <div>
                    <label for="files" class="drop-container" id="dropcontainer">
                        <span class="drop-title">{"Drop files here"}</span>
                        {"or"}
                        <input type="file" id="files" onchange={on_submit} multiple=true/>
                    </label>
                </div>
                <div>
                    {for self.decryted_files.iter().map(|f| f.view())}
                </div>

            </div>
        }
    }
}

impl DecryptTask {
    fn view(&self) -> Html {
        match self {
            DecryptTask::Decrypting(name) => html! {
                <div class="fileblock">
                    <MatCircularProgress indeterminate=true />
                    <p>{format!("Decrypting {name} ...")}</p>
                </div>
            },
            DecryptTask::Finished(df) => html! {
                html! {
                    <div class="fileblock">
                        {df.view()}
                    </div>
                }
            },
            DecryptTask::Error(name, e) => html! {
                <div class="fileblock">
                    <p>{format!("{name} ERROR: {e}")}</p>
                </div>
            },
        }
    }
}

impl DecryptedFile {
    fn view(&self) -> Html {
        // let img = self
        //     .image
        //     .as_ref()
        //     .map(|(format, image)| img_to_element(*format, image.as_str()));
        html! {
            <div>
                <div>
                    {format!("{} 👉", self.original_name)}
                    <a download={self.name.clone()} href={self.object_url.to_string()}>
                        {self.name.clone()}
                    </a>
                </div>
                if let Some(info) = self.info.as_ref() {
                    <p>{format!("{}", info.name)}</p>
                    <p>{
                        format!("Artist: {}", info.artist.iter().map(|(s, _)| s.as_str()).collect::<Vec<&str>>().join(","))
                    }</p>
                }
            </div>
        }
    }
}
