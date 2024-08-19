// SPDX-License-Identifier: MPL-2.0
// 2024 - Dominic Gerhauser and contributors

use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};

use crate::window::Window;

use config::{Annotation, Config, CONFIG_VERSION};
use cosmic::cosmic_config;
use cosmic::cosmic_config::CosmicConfigEntry;
mod config;
use window::Flags;

mod localize;
mod widget_copy;
mod window;

fn main() -> cosmic::iced::Result {
    localize::localize();

    let mut annotations = HashMap::new();
    let xdg_data_dir = env::var("XDG_DATA_DIRS").unwrap_or_else(|e| {
        eprintln!("failed to read `XDG_DATA_DIRS`: {e}");
        "/usr/share:/usr/locale/share".to_string()
    });
    let xdg_data_dir = xdg_data_dir.split(':').find(|path| {
        let id_path: PathBuf = [path, window::ID, "i18n-json"].iter().collect();
        id_path.exists() && id_path.is_dir()
    });
    let mut requested_languages: Vec<_> =
        i18n_embed::DesktopLanguageRequester::requested_languages();
    let requested_languages = fluent_langneg::convert_vec_str_to_langids_lossy(
        requested_languages.drain(..).map(|lang| lang.to_string()),
    );

    let default_language: fluent_langneg::LanguageIdentifier = "en".parse().unwrap();
    if let Some(dir) = xdg_data_dir {
        let i18n_json_dir: PathBuf = [dir, window::ID, "i18n-json"].iter().collect();
        let locales_in_dir = match fs::read_dir(i18n_json_dir) {
            Ok(dir_iter) => dir_iter
                .filter_map(|file_res| match file_res {
                    Ok(file) => Some(
                        file.file_name()
                            .to_str()
                            .expect("filename is invalid utf8")
                            .to_string(),
                    ),
                    Err(err) => {
                        eprintln!("could not read file: {err}");
                        None
                    }
                })
                .collect(),
            Err(err) => {
                eprintln!("could not read directory: {dir}: err: {err}",);
                Vec::new()
            }
        };

        let available_languages = fluent_langneg::convert_vec_str_to_langids_lossy(locales_in_dir);
        let supported_languages = fluent_langneg::negotiate_languages(
            &requested_languages,
            &available_languages,
            Some(&default_language),
            fluent_langneg::NegotiationStrategy::Filtering,
        );
        for lang_code in supported_languages.into_iter().rev() {
            let lang_code = lang_code.to_string();
            let annotation_file: PathBuf =
                [dir, window::ID, "i18n-json", &lang_code, "annotations.json"]
                    .iter()
                    .collect();
            let file_contents = match fs::read(&annotation_file) {
                Ok(ok) => ok,
                Err(e) => {
                    eprintln!("could not read annotations.json file: {annotation_file:?} - {lang_code} - {e}");
                    continue;
                }
            };

            let annotations_locale: HashMap<String, Annotation> = match serde_json::from_slice(
                &file_contents,
            ) {
                Ok(ok) => ok,
                Err(e) => {
                    eprintln!("could not parse annotations.json file: {annotation_file:?} {lang_code} - {e}");
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
