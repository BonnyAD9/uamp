use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File},
    path::{Path, PathBuf},
};

use crate::err::Result;

/// Configuration of uamp
#[derive(Clone, Serialize, Deserialize)]
#[serde(
    tag = "$schema",
    rename = "https://raw.githubusercontent.com/BonnyAD9/uamp/master/other/json_schema/config_schema.json"
)]
pub struct Config {
    /// Where the configuration should be saved
    #[serde(skip_serializing, default = "default_config_path_json")]
    config_path: PathBuf,
    /// Where to search for songs
    #[serde(default = "default_search_paths")]
    pub search_paths: Vec<PathBuf>,
    /// Determines wheter the search for songs should be recursive
    #[serde(default = "default_recursive_search")]
    pub recursive_search: bool,
    /// Where to save/load library from
    #[serde(default = "default_library_path")]
    pub library_path: PathBuf,
    /// Where to save/load player state from
    #[serde(default = "default_player_path")]
    pub player_path: PathBuf,
    /// What are the file extensions for audio files
    #[serde(default = "default_audio_extensions")]
    pub audio_extensions: Vec<String>,
    /// When true, library will search for new songs on startup
    #[serde(default = "default_update_library_on_start")]
    pub update_library_on_start: bool,
    /// When true, uamp will register global shortcuts
    #[serde(default = "default_register_global_hotkeys")]
    pub register_global_hotkeys: bool,
    /// Determines how much should the volume change with volume up/down
    #[serde(default = "default_volume_jump")]
    pub volume_jump: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config::new(default_config_path())
    }
}

impl Config {
    /// Loads config from the given json file. If the loading fails, creates
    /// default config.
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

    /// Loads config from the default json file. If the loading fails, creates
    /// default config.
    pub fn from_default_json() -> Self {
        Config::from_json(default_config_path().join("config.json"))
    }

    /// Saves the config to the given json file.
    ///
    /// # Errors
    /// - Fails to create parent directory
    /// - Fails to write to file
    /// - Fails to serialize
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

    /// Saves the config to the default json file.
    ///
    /// # Errors
    /// - Fails to create parent directory
    /// - Fails to write to fi
    pub fn to_default_json(&self) -> Result<()> {
        self.to_json(&self.config_path)
    }

    /// Creates new config with the given config path
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        Config {
            config_path: config_path.into(),
            search_paths: default_search_paths(),
            recursive_search: default_recursive_search(),
            library_path: default_library_path(),
            player_path: default_player_path(),
            audio_extensions: default_audio_extensions(),
            update_library_on_start: default_update_library_on_start(),
            register_global_hotkeys: default_register_global_hotkeys(),
            volume_jump: default_volume_jump(),
        }
    }
}

/// Gets the unique app identifier, it is different when debugging
pub fn app_id() -> String {
    #[cfg(not(debug_assertions))]
    {
        "uamp".to_owned()
    }
    #[cfg(debug_assertions)]
    {
        "uamp_debug".to_owned()
    }
}

/// Gets the default path for configuration, it is different when debugging
pub fn default_config_path() -> PathBuf {
    if let Some(dir) = dirs::config_dir() {
        // use different path when debugging to not ruin existing config
        #[cfg(not(debug_assertions))]
        {
            dir.join("uamp")
        }
        #[cfg(debug_assertions)]
        {
            dir.join("uamp_debug")
        }
    } else {
        PathBuf::from(".")
    }
}

/// Gets the default path to json configuration, it is different when debugging
fn default_config_path_json() -> PathBuf {
    default_config_path().join("config.json")
}

/// Gets the default search paths.
fn default_search_paths() -> Vec<PathBuf> {
    if let Some(dir) = dirs::audio_dir() {
        vec![dir]
    } else {
        vec![PathBuf::from(".")]
    }
}

/// Gets the default value for recursive_search
fn default_recursive_search() -> bool {
    true
}

/// Gets the default path to library json file, it is different when debugging
fn default_library_path() -> PathBuf {
    default_config_path().join("library.json")
}

/// Gets the default path to player state json file, it is different when
/// debugging
fn default_player_path() -> PathBuf {
    default_config_path().join("player.json")
}

/// Gets the default audio file extensions
fn default_audio_extensions() -> Vec<String> {
    vec![
        "flac".to_owned(),
        "mp3".to_owned(),
        "m4a".to_owned(),
        "mp4".to_owned(),
    ]
}

/// Gets the default value for update_library_on_start
fn default_update_library_on_start() -> bool {
    false
}

/// Gets the default value for register_global_hotkeys
fn default_register_global_hotkeys() -> bool {
    false
}

/// Gets the default port for the server, it is defferent when debugging
pub fn default_port() -> u16 {
    #[cfg(not(debug_assertions))]
    {
        8267
    }
    #[cfg(debug_assertions)]
    {
        33284
    }
}

/// Gets the default volume jump.
fn default_volume_jump() -> f32 {
    0.025
}
