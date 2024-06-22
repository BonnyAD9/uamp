use serde::{Deserialize, Serialize};
use std::{cell::Cell, collections::HashMap, path::PathBuf, time::Duration};

use crate::{
    core::{config::default_config_path_json, control_msg_vec::ControlMsgVec},
    ext::Wrap,
    gen_struct,
};

use super::{default_config_path, song_pos_save::SongPosSave};

gen_struct! {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(
        tag = "$schema",
        rename = "https://raw.githubusercontent.com/BonnyAD9/uamp/master/other/json_schema/config_schema.json"
    )]
    pub Config {
        // Fields passed by reference

        #[doc = "Folders where to look for songs."]
        search_paths: Vec<PathBuf> { pub, pub } => pub(super) () {
            if let Some(dir) = dirs::audio_dir() {
                vec![dir]
            } else {
                vec![PathBuf::from(".")]
            }
        },

        #[doc = "Path to library data file."]
        library_path: Option<PathBuf> { pub, pub } => pub(super) () {
            Some(default_config_path().join("library.json"))
        },

        #[doc = "Path to player data file."]
        player_path: Option<PathBuf> { pub, pub } => pub(super) () {
            Some(default_config_path().join("player.json"))
        },

        #[doc = "File extensions that will be used to recognize audio files."]
        audio_extensions: Vec<String> { pub, pub } => pub(super) () {
            vec![
                "flac".to_owned(),
                "mp3".to_owned(),
                "m4a".to_owned(),
                "mp4".to_owned(),
            ]
        },

        #[doc = "Address of server that is used to control uamp."]
        server_address: String { pub, pub } => pub(super) () {
            "127.0.0.1".to_owned()
        },

        #[doc = "Aliases for groups of control actions."]
        control_aliases: HashMap<String, ControlMsgVec> { pub, pub } => pub(super) () {
            Default::default()
        },

        ; // fields passed by value:

        #[doc = "When enabled uamp will sort only by the primary attribute."]
        simple_sorting: bool { pub, pub } => pub(super) () false,

        #[doc = "When enabled, uamp will start playing immidietly when it"]
        #[doc = "starts."]
        play_on_start: bool { pub, pub } => pub(super) () false,

        #[doc = "When disabled the currently playing song will be inserted to"]
        #[doc = "the first position in playlist after shuffling."]
        shuffle_current: bool { pub, pub } => pub(super) () true,

        #[doc = "Determines whether to recursively traverse directories when"]
        #[doc = "searching for new songs."]
        recursive_search: bool { pub, pub } => pub(super) () true,

        #[doc = "When enabled, uamp will automatically look for new songs"]
        #[doc = "immidietly when it starts."]
        update_library_on_start: bool { pub, pub } => pub(super) () true,

        #[doc = "When enbled, non existing songs will be removed from library"]
        #[doc = "when looking for new songs."]
        remove_missing_on_load: bool { pub, pub } => pub(super) () true,

        #[doc = "Determines how much the volumes changes with volume up/down"]
        #[doc = "message."]
        volume_jump: f32 { pub, pub } => pub(super) () 0.025,

        #[doc = "Determines whether the playback position is saved."]
        save_playback_pos: SongPosSave { pub, pub } => pub(super) () {
            SongPosSave::Never
        },

        #[doc = "Determines how often uamp automatically saves its state."]
        save_timeout: Option<Wrap<Duration>> { pub, pub } => pub(super) () {
            Some(Wrap(Duration::from_secs(60)))
        },

        #[doc = "Sets length of the volume fade of song on play/pause."]
        fade_play_pause: Wrap<Duration> { pub, pub } => pub(super) () {
            Wrap(Duration::from_millis(150))
        },

        #[doc = "Enable/Disable gapless playback."]
        gapless: bool { pub, pub } => pub(super) () false,

        #[doc = "Detemines how much uamp seeks with fast forward/rewind"]
        seek_jump: Wrap<Duration> { pub, pub } => pub(super) () {
            Wrap(Duration::from_secs(10))
        },

        #[doc = "The port of the server that is used to control uamp."]
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

        #[doc = "Determines how old must logs be so that they are"]
        #[doc = "automatically deleted."]
        delete_logs_after: Wrap<Duration> { pub, pub } => pub(super) () {
            // 3 days
            Wrap(Duration::from_secs(60 * 60 * 24 * 3))
        },

        #[doc = "Enable/Disable server that is used to control uamp. Server"]
        #[doc = "is sometimes forced to be enabled and so this has somtimes"]
        #[doc = "no effect."]
        enable_server: bool { pub, pub } => pub(super) () true,

        #[doc = "When jumping to the start of the song, if the command is"]
        #[doc = "issued again within this time, previous song will be played."]
        previous_timeout: Option<Wrap<Duration>> { pub, pub }
            => pub(super) () None,

        ; // fields that aren't serialized

        #[serde(skip_serializing, default = "default_config_path_json")]
        pub config_path: Option<PathBuf>,
        #[serde(skip_serializing, default)]
        pub force_server: bool,

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
            audio_extensions: default_audio_extensions(),
            update_library_on_start: default_update_library_on_start(),
            remove_missing_on_load: default_remove_missing_on_load(),
            volume_jump: default_volume_jump(),
            save_playback_pos: default_save_playback_pos(),
            save_timeout: default_save_timeout(),
            fade_play_pause: default_fade_play_pause(),
            gapless: default_gapless(),
            seek_jump: default_seek_jump(),
            port: default_port(),
            server_address: default_server_address(),
            control_aliases: default_control_aliases(),
            delete_logs_after: default_delete_logs_after(),
            enable_server: default_enable_server(),
            shuffle_current: default_shuffle_current(),
            previous_timeout: default_previous_timeout(),
            play_on_start: default_play_on_start(),
            simple_sorting: default_simple_sorting(),
            force_server: false,
            change: Cell::new(true),
        }
    }

    /// Sets the change to the given value. Use with caution.
    pub(super) fn set_change(&self, value: bool) {
        self.change.set(value);
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new(Some(default_config_path()))
    }
}
