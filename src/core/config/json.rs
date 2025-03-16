use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use log::{error, info};
use serde::Serialize;

use crate::core::{Error, Result};

use super::{Config, default_config_dir};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

impl Config {
    /// Loads config from the default json file. If the loading fails, creates
    /// default config.
    pub fn from_default_json() -> Self {
        match Config::from_json(default_config_dir().join("config.json")) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to load config: {}", e.log());
                Config::default()
            }
        }
    }

    /// Loads config from the given json file. If the loading fails, creates
    /// default config.
    ///
    /// # Errors
    /// - The config fails to parse.
    pub fn from_json(path: impl AsRef<Path>) -> Result<Self> {
        let file = match File::open(path.as_ref()) {
            Ok(f) => f,
            Err(_) => {
                info!(
                    "the config file {:?} doesn't exist, creating default",
                    path.as_ref()
                );
                let conf = Config::new(Some(path.as_ref()));
                if let Err(e) = conf.to_default_json() {
                    error!(
                        "failed to save config to file {:?}: {}",
                        path.as_ref(),
                        e.log()
                    );
                }
                return Ok(conf);
            }
        };

        let mut conf: Self = serde_json::from_reader(file).map_err(|e| {
            Error::SerdeJson(e.into()).msg("Failed to load config from json.")
        })?;
        conf.config_path = Some(path.as_ref().to_owned());
        Ok(conf)
    }

    /// Saves the config to the default json file. Doesn't save if there was no
    /// chagnge since the last save.
    ///
    /// # Errors
    /// - Fails to create parent directory
    /// - Fails to write to fi
    pub fn to_default_json(&self) -> Result<()> {
        if self.changed() {
            if let Some(p) = &self.config_path {
                self.to_json_file(p)?;
            }
            self.set_change(false);
        }
        Ok(())
    }

    /// Saves the config to the given json file.
    ///
    /// # Errors
    /// - Fails to create parent directory
    /// - Fails to write to file
    /// - Fails to serialize
    pub fn to_json_file(&self, path: impl AsRef<Path>) -> Result<()> {
        if let Some(par) = path.as_ref().parent() {
            fs::create_dir_all(par)?;
        }

        self.to_json(File::create(&path)?)
    }

    pub fn to_json(&self, w: impl Write) -> Result<()> {
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut ser = serde_json::Serializer::with_formatter(w, formatter);
        self.serialize(&mut ser)?;
        Ok(())
    }
}
