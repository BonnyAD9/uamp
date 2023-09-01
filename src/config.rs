use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    cell::Cell,
    fs::{create_dir_all, File},
    path::{Path, PathBuf},
};

use crate::{err::Result, gen_struct};

gen_struct! {
    #[derive(Clone)]
    #[serde(
        tag = "$schema",
        rename = "https://raw.githubusercontent.com/BonnyAD9/uamp/master/other/json_schema/config_schema.json"
    )]
    pub Config {
        // Fields passed by reference

        search_paths: Vec<PathBuf> { pub, pub }
        fn "default_search_paths": {
            if let Some(dir) = dirs::audio_dir() {
                vec![dir]
            } else {
                vec![PathBuf::from(".")]
            }
        },

        library_path: PathBuf { pub, pub }
        fn "default_library_path": default_config_path().join("library.json"),

        player_path: PathBuf { pub, pub }
        fn "default_player_path": default_config_path().join("player.json"),

        audio_extensions: Vec<String> { pub, pub }
        fn "default_audio_extensions": {
            vec![
                "flac".to_owned(),
                "mp3".to_owned(),
                "m4a".to_owned(),
                "mp4".to_owned(),
            ]
        },

        ; // fields passed by value:

        recursive_search: bool { pub, pub }
        fn "default_recursive_search": true,

        update_library_on_start: bool { pub, pub }
        fn "default_update_library_on_start": true,

        register_global_hotkeys: bool { pub, pub }
        fn "default_register_global_hotkeys": true,

        volume_jump: f32 { pub, pub }
        fn "default_volume_jump": 0.025

        ; // fields that aren't serialized

        #[serde(skip_serializing, default = "default_config_path_json")]
        config_path: PathBuf,
    }
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

    /// Saves the config to the default json file. Doesn't save if there was no
    /// chagnge since the last save.
    ///
    /// # Errors
    /// - Fails to create parent directory
    /// - Fails to write to fi
    pub fn to_default_json(&self) -> Result<()> {
        self.to_json(&self.config_path)?;
        self.change.set(false);
        Ok(())
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
            change: Cell::new(true),
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
