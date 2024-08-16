use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs, io};

use crate::window::Window;

use config::{Annotation, Config, CONFIG_VERSION};
use cosmic::cosmic_config;
use cosmic::cosmic_config::CosmicConfigEntry;
mod config;
use localize::LANGUAGE_LOADER;
use window::Flags;

mod localize;
mod style_copy;
mod widget_copy;
mod window;

fn main() -> cosmic::iced::Result {
    localize::localize();

    let current_languages: Vec<_> = LANGUAGE_LOADER
        .current_languages()
        .into_iter()
        .map(|l| l.to_string())
        .collect();
    let mut annotations = HashMap::new();
    let xdg_data_dir = env::var("XDG_DATA_DIRS").unwrap_or_else(|e| {
        eprintln!("failed to read `XDG_DATA_DIRS`: {e}");
        "/usr/share:/usr/locale/share".to_string()
    });
    let xdg_data_dir = xdg_data_dir.split(':').find(|path| {
        let id_path: PathBuf = [path, window::ID, "i18n-json"].iter().collect();
        id_path.exists() && id_path.is_dir()
    });
    if let Some(dir) = xdg_data_dir {
        for lang_code in current_languages.into_iter().rev() {
            let file_path = if cfg!(feature = "compress") {
                "annotations.json.xz"
            } else {
                "annotations.json"
            };
            #[cfg(feature = "compress")]
            let mut is_compressed = true;
            let annotation_file_path: PathBuf =
                [dir, window::ID, "i18n-json", &lang_code, file_path]
                    .iter()
                    .collect();
            let mut annotation_file_reader: Box<dyn io::Read> = match fs::File::open(
                &annotation_file_path,
            ) {
                Ok(reader) => Box::new(reader),
                Err(out_err) => {
                    #[cfg(feature = "compress")]
                    {
                        let annotation_file_path: PathBuf =
                            [dir, window::ID, "i18n-json", &lang_code, "annotations.json"]
                                .iter()
                                .collect();
                        match fs::File::open(&annotation_file_path) {
                            Ok(reader) => {
                                is_compressed = false;
                                Box::new(reader)
                            }
                            Err(inner_err) => {
                                eprintln!("could not read annotations.json and annotations.json.xz files with compress feature enabled: {annotation_file_path:?} - {lang_code} - {out_err} {inner_err}");
                                continue;
                            }
                        }
                    }
                    #[cfg(not(feature = "compress"))]
                    {
                        eprintln!("could not read annotations.json file: {annotation_file_path:?} - {lang_code} - {out_err}");
                        continue;
                    }
                }
            };
            annotation_file_reader = Box::new(io::BufReader::new(annotation_file_reader));
            #[cfg(feature = "compress")]
            if is_compressed {
                annotation_file_reader =
                    Box::new(liblzma::read::XzDecoder::new(annotation_file_reader));
            }
            let annotations_locale: HashMap<String, Annotation> = match serde_json::from_reader(
                annotation_file_reader,
            ) {
                Ok(ok) => ok,
                Err(e) => {
                    eprintln!("could not parse annotations.json file: {annotation_file_path:?} - {lang_code} - {e}");
                    continue;
                }
            };

            annotations.extend(annotations_locale);
        }
    }

    let (config_handler, config) = match cosmic_config::Config::new(window::ID, CONFIG_VERSION) {
        Ok(config_handler) => {
            let config = match Config::get_entry(&config_handler) {
                Ok(ok) => ok,
                Err((errs, config)) => {
                    eprintln!("errors loading config: {:?}", errs);
                    config
                }
            };
            (Some(config_handler), config)
        }
        Err(err) => {
            eprintln!("failed to create config handler: {}", err);
            (None, Config::default())
        }
    };
    let flags = Flags {
        config_handler,
        config,
        annotations,
    };
    cosmic::applet::run::<Window>(true, flags)
}
