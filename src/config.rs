use eyre::{Report, Result};
use serde_derive::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File},
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_library_path")]
    library_path: PathBuf,
}

fn default_library_path() -> PathBuf {
    if let Some(dir) = dirs::audio_dir() {
        dir
    } else {
        PathBuf::from("./")
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            library_path: default_library_path(),
        }
    }
}

impl Config {
    pub fn library_path(&self) -> &PathBuf {
        &self.library_path
    }

    pub fn from_json<P: AsRef<Path>>(path: P) -> Self {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return Config::default(),
        };

        serde_json::from_reader(file).unwrap_or_default()
    }

    pub fn from_default_json<P: AsRef<Path>>(path: P) -> Self {
        if let Some(dir) = default_config_path() {
            Config::from_json(dir.join("/config.json"))
        } else {
            Config::default()
        }
    }

    pub fn to_json<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        if let Some(par) = path.as_ref().parent() {
            create_dir_all(par)?;
        }

        serde_json::to_writer(File::create(path)?, self)?;
        Ok(())
    }

    pub fn to_default_json(&self) -> Result<()> {
        let path = default_config_path()
            .ok_or(Report::msg("Couldn't get the default path"))?;
        self.to_json(path)
    }
}

fn default_config_path() -> Option<PathBuf> {
    if let Some(mut dir) = dirs::config_dir() {
        Some(dir.join("/uamp"))
    } else {
        None
    }
}
