use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    cell::Cell,
    collections::HashMap,
    fs::{create_dir_all, read_dir, remove_file, File},
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    core::{extensions::Wrap, Result},
    gen_struct,
};

gen_struct! {
    #[derive(Clone, Serialize, Deserialize)]
    #[serde(
        tag = "$schema",
        rename = "https://raw.githubusercontent.com/BonnyAD9/uamp/master/other/json_schema/config_schema.json"
    )]
    pub Config {
        // Fields passed by reference

        search_paths: Vec<PathBuf> { pub, pub } => pub(super) () {
            if let Some(dir) = dirs::audio_dir() {
                vec![dir]
            } else {
                vec![PathBuf::from(".")]
            }
        },

        library_path: Option<PathBuf> { pub, pub } => pub(super) () {
            Some(default_config_path().join("library.json"))
        },

        player_path: Option<PathBuf> { pub, pub } => pub(super) () {
            Some(default_config_path().join("player.json"))
        },

        gui_state_path: Option<PathBuf> { pub, pub } => pub(super) () {
            Some(default_config_path().join("gui_state.json"))
        },

        image_cache: Option<PathBuf> { pub, pub } => pub(super) () {
            Some(default_config_path().join("img_cache"))
        },

        audio_extensions: Vec<String> { pub, pub } => pub(super) () {
            vec![
                "flac".to_owned(),
                "mp3".to_owned(),
                "m4a".to_owned(),
                "mp4".to_owned(),
            ]
        },

        global_hotkeys: HashMap<String, String> { pub, pub } => pub(super) () {
            let mut hm = HashMap::new();
            hm.insert("ctrl+alt+home".to_owned(), "pp".to_owned());
            hm.insert("ctrl+alt+pg_down".to_owned(), "ns".to_owned());
            hm.insert("ctrl+alt+pg_up".to_owned(), "ps".to_owned());
            hm.insert("ctrl+alt+up".to_owned(), "vu".to_owned());
            hm.insert("ctrl+alt+down".to_owned(), "vd".to_owned());
            hm.insert("ctrl+alt+left".to_owned(), "rw".to_owned());
            hm.insert("ctrl+alt+right".to_owned(), "ff".to_owned());
            hm
        },

        server_address: String { pub, pub } => pub(super) () {
            "127.0.0.1".to_owned()
        },

        ; // fields passed by value:

        simple_sorting: bool { pub, pub } => pub(super) () false,

        play_on_start: bool { pub, pub } => pub(super) () false,

        show_help: bool { pub, pub } => pub(super) () true,

        shuffle_current: bool { pub, pub } => pub(super) () true,

        recursive_search: bool { pub, pub } => pub(super) () true,

        update_library_on_start: bool { pub, pub } => pub(super) () true,

        register_global_hotkeys: bool { pub, pub } => pub(super) () false,

        volume_jump: f32 { pub, pub } => pub(super) () 0.025,

        save_timeout: Option<Wrap<Duration>> { pub, pub } => pub(super) () {
            Some(Wrap(Duration::from_secs(60)))
        },

        fade_play_pause: Wrap<Duration> { pub, pub } => pub(super) () {
            Wrap(Duration::from_millis(150))
        },

        gapless: bool { pub, pub } => pub(super) () false,

        tick_length: Wrap<Duration> { pub, pub } => pub(super) () {
            Wrap(Duration::from_secs(1))
        },

        seek_jump: Wrap<Duration> { pub, pub } => pub(super) () {
            Wrap(Duration::from_secs(10))
        },

        port: u16 { pub, pub } => pub(super) () {
            #[cfg(not(debug_assertions))]
            {
                8267
            }
            #[cfg(debug_assertions)]
            {
                33284
            }
        },

        delete_logs_after: Wrap<Duration> { pub, pub } => pub(super) () {
            // 3 days
            Wrap(Duration::from_secs(60 * 60 * 24 * 3))
        },

        enable_server: bool { pub, pub } => pub(super) () true,

        previous_timeout: Option<Wrap<Duration>> { pub, pub }
            => pub(super) () None,

        show_remaining_time: bool { pub, pub } => pub(super) () false,

        ; // fields that aren't serialized

        #[serde(skip_serializing, default = "default_config_path_json")]
        pub config_path: Option<PathBuf>,

        ; // attributes for the auto field
        #[serde(skip)]
    }
}

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

impl Config {
    /// Returns true if some of the saved data changed from the last save.
    pub fn changed(&self) -> bool {
        self.change.get()
    }

    /// Loads config from the default json file. If the loading fails, creates
    /// default config.
    pub fn from_default_json() -> Self {
        Config::from_json(default_config_path().join("config.json"))
    }

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
                let conf = Config::new(Some(path.as_ref()));
                if let Err(e) = conf.to_default_json() {
                    error!(
                        "failed to save config to file {:?}: {e}",
                        path.as_ref()
                    );
                }
                return conf;
            }
        };

        match serde_json::from_reader(file) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to load config file: {e}");
                Config::default()
            }
        }
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
                self.to_json(p)?;
            }
            self.change.set(false);
        }
        Ok(())
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

    /// Creates new config with the given config path
    pub fn new<P>(config_path: Option<P>) -> Self
    where
        P: Into<PathBuf>,
    {
        Config {
            config_path: config_path.map(|p| p.into()),
            search_paths: default_search_paths(),
            recursive_search: default_recursive_search(),
            library_path: default_library_path(),
            player_path: default_player_path(),
            gui_state_path: default_gui_state_path(),
            image_cache: default_image_cache(),
            audio_extensions: default_audio_extensions(),
            update_library_on_start: default_update_library_on_start(),
            register_global_hotkeys: default_register_global_hotkeys(),
            volume_jump: default_volume_jump(),
            save_timeout: default_save_timeout(),
            fade_play_pause: default_fade_play_pause(),
            global_hotkeys: default_global_hotkeys(),
            gapless: default_gapless(),
            tick_length: default_tick_length(),
            seek_jump: default_seek_jump(),
            port: default_port(),
            server_address: default_server_address(),
            delete_logs_after: default_delete_logs_after(),
            enable_server: default_enable_server(),
            shuffle_current: default_shuffle_current(),
            show_help: default_show_help(),
            previous_timeout: default_previous_timeout(),
            show_remaining_time: default_show_remaining_time(),
            play_on_start: default_play_on_start(),
            simple_sorting: default_simple_sorting(),
            change: Cell::new(true),
        }
    }

    pub fn delete_old_logs(&self) -> Result<()> {
        let dir = read_dir(default_config_path().join("log"))?;

        for d in dir {
            let d = d?;
            let mt = d.metadata()?.modified()?;
            if mt.elapsed()? > self.delete_logs_after().0 {
                remove_file(d.path())?;
            }
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new(Some(default_config_path()))
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

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

/// Gets the default path to json configuration, it is different when debugging
fn default_config_path_json() -> Option<PathBuf> {
    Some(default_config_path().join("config.json"))
}
