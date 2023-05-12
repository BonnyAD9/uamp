use std::fs::File;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_library_path")]
    library_path: String,
}

fn default_library_path() -> String {
    if let Some(dir) = dirs::audio_dir() {
        if let Some(dir) = dir.to_str() {
            return dir.to_owned()
        }
    }

    "./".to_owned()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            library_path: default_library_path(),
        }
    }
}

impl Config {
    pub fn library_path(&self) -> &str {
        &self.library_path
    }

    pub fn from_json(path: &str) -> Self {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return Config::default(),
        };

        serde_json::from_reader(file).unwrap_or_default()
    }

    pub fn from_default_json(path: &str) -> Self {
        if let Some(dir) = dirs::config_dir() {
            if let Some(dir) = dir.to_str() {
                return Config::from_json(dir);
            }
        }

        Config::default()
    }
}
