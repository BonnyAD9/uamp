use eyre::Result;
use log::{error, info};
use serde::Serialize;
use serde_derive::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File},
    path::{Path, PathBuf},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing, default = "default_config_path")]
    config_path: PathBuf,
    #[serde(default = "default_search_paths")]
    pub search_paths: Vec<PathBuf>,
    #[serde(default = "default_library_path")]
    pub library_path: PathBuf,
    #[serde(default = "default_audio_extensions")]
    pub audio_extensions: Vec<String>,
    #[serde(default = "default_update_library_on_start")]
    pub update_library_on_start: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config::new(default_config_path())
    }
}

impl Config {
    pub fn from_json<P: AsRef<Path>>(path: P) -> Self {
        let file = match File::open(path.as_ref()) {
            Ok(f) => f,
            Err(_) => {
                info!(
                    "the config file {:?} doesn't exist, creating default",
                    path.as_ref()
                );
                let conf = Config::new(path.as_ref());
                if let Err(e) = conf.to_default_json() {
                    error!(
                        "failed to save config to file {:?}: {e}",
                        path.as_ref()
                    );
                }
                return conf;
            }
        };

        serde_json::from_reader(file).unwrap_or_default()
    }

    pub fn from_default_json() -> Self {
        Config::from_json(default_config_path().join("config.json"))
    }

    pub fn to_json<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        if let Some(par) = path.as_ref().parent() {
            create_dir_all(par)?;
        }

        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut ser = serde_json::Serializer::with_formatter(
            File::create(path)?,
            formatter,
        );
        self.serialize(&mut ser)?;

        Ok(())
    }

    pub fn to_default_json(&self) -> Result<()> {
        self.to_json(&self.config_path)
    }

    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        Config {
            config_path: config_path.into(),
            search_paths: default_search_paths(),
            library_path: default_library_path(),
            audio_extensions: default_audio_extensions(),
            update_library_on_start: default_update_library_on_start(),
        }
    }
}

pub fn default_config_path() -> PathBuf {
    if let Some(dir) = dirs::config_dir() {
        dir.join("uamp")
    } else {
        PathBuf::from(".")
    }
}

fn default_search_paths() -> Vec<PathBuf> {
    if let Some(dir) = dirs::audio_dir() {
        vec![dir]
    } else {
        vec![PathBuf::from(".")]
    }
}

fn default_library_path() -> PathBuf {
    default_config_path().join("library.json")
}

fn default_audio_extensions() -> Vec<String> {
    vec![
        "flac".to_owned(),
        "mp3".to_owned(),
        "m4a".to_owned(),
        "mp4".to_owned(),
    ]
}

fn default_update_library_on_start() -> bool {
    false
}
