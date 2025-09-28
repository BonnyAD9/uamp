use serde::{Deserialize, Serialize};
use std::{cell::Cell, collections::HashMap, path::PathBuf, time::Duration};
use uamp_proc::{JsonValueUpdate, PartialClone, TrackChange};

use crate::{
    core::{
        Alias, ControlFunction, Error, Result,
        config::{Change, default},
    },
    env::{RunType, install},
    ext::Wrap,
};

use super::{CacheSize, song_pos_save::SongPosSave};

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    JsonValueUpdate,
    PartialClone,
    TrackChange,
)]
#[serde(
    tag = "$schema",
    rename = "https://raw.githubusercontent.com/BonnyAD9/uamp/master/other/json_schema/config_schema.json"
)]
#[json_value_update(
    Result<Change>,
    k => return Err(Error::invalid_value(format!("Invalid setting key `{k}`."))),
    Change::empty(),
)]
#[partial_clone(
    pub,
    ConfigClone,
    #[derive(Debug, Clone, Serialize, Deserialize)]
)]
pub struct Config {
    // Fields passed by reference
    /// Folders where to look for songs.
    #[track_ref(pub, pub)]
    #[serde(default = "default::search_paths")]
    search_paths: Vec<PathBuf>,

    /// Path to library data file.
    #[track_ref(pub, pub)]
    #[serde(default = "default::library_path")]
    #[value_change(Change::LIBRARY_PATH)]
    library_path: Option<PathBuf>,

    /// Path to player data file.
    #[track_ref(pub, pub)]
    #[serde(default = "default::player_path")]
    #[value_change(Change::PLAYER_PATH)]
    player_path: Option<PathBuf>,

    /// Path to uamp cache folder
    #[track_ref(pub, pub)]
    #[serde(default = "default::cache_path")]
    #[value_change(Change::CACHE_PATH)]
    cache_path: PathBuf,

    /// File extensions that will be used to recognize audio files.
    #[track_ref(pub, pub)]
    #[serde(default = "default::audio_extensions")]
    #[value_change(Change::AUDIO_EXTENSIONS)]
    audio_extensions: Vec<String>,

    /// Address of server that is used to control uamp.
    #[track_ref(pub, pub)]
    #[serde(default = "default::server_address")]
    #[value_change(Change::SERVER_ADDRESS)]
    server_address: String,

    /// Aliases for groups of control actions.
    #[track_ref(pub, pub)]
    #[serde(default = "default::control_aliases")]
    control_aliases: HashMap<String, ControlFunction>,

    /// This will be used as playlist end action if the end action
    /// is not set.
    #[track_ref(pub, pub)]
    #[serde(default)]
    default_playlist_end_action: Option<Alias>,

    /// Determines how uamp will self update.
    #[track_ref(pub, pub)]
    #[serde(default)]
    update_mode: install::UpdateMode,

    /// Determines the repository fro which uamp will update.
    #[track_ref(pub, pub)]
    #[serde(default = "default::update_remote")]
    update_remote: String,

    #[track_ref(pub, pub)]
    #[serde(default = "default::skin")]
    #[value_change(Change::SKIN)]
    skin: PathBuf,

    #[track_ref(pub, pub)]
    #[serde(default)]
    web_client_command: Option<String>,

    #[track_ref(pub, pub)]
    #[serde(default = "default::plugin_folders")]
    plugin_folders: Vec<PathBuf>,

    // fields passed by value:
    /// When enabled uamp will sort only by the primary attribute.
    #[track_value(pub, pub, eq)]
    #[serde(default)]
    simple_sorting: bool,

    /// When enabled, uamp will start playing immidietly when it
    /// starts.
    #[track_value(pub, pub, eq)]
    #[serde(default)]
    play_on_start: bool,

    /// When disabled the currently playing song will be inserted to
    /// the first position in playlist after shuffling.
    #[track_value(pub, pub, eq)]
    #[serde(default = "default::shuffle_current")]
    shuffle_current: bool,

    /// Determines whether to recursively traverse directories when
    /// searching for new songs.
    #[track_value(pub, pub, eq)]
    #[serde(default = "default::recursive_search")]
    #[value_change(Change::RECURSIVE_SEARCH)]
    recursive_search: bool,

    /// When enabled, uamp will automatically look for new songs
    /// immidietly when it starts.
    #[track_value(pub, pub, eq)]
    #[serde(default = "default::update_library_on_start")]
    update_library_on_start: bool,

    /// When enbled, non existing songs will be removed from library
    /// when looking for new songs.
    #[track_value(pub, pub, eq)]
    #[serde(default = "default::remove_missing_on_load")]
    remove_missing_on_load: bool,

    /// Determines how much the volumes changes with volume up/down
    /// message.
    #[track_value(pub, pub, eq)]
    #[serde(default = "default::volume_jump")]
    volume_jump: f32,

    /// Determines whether the playback position is saved.
    #[track_value(pub, pub, eq)]
    #[serde(default = "default::save_playback_pos")]
    save_playback_pos: SongPosSave,

    /// Determines how often uamp automatically saves its state.
    #[track_value(pub, pub, eq)]
    #[serde(default = "default::save_timeout")]
    save_timeout: Option<Wrap<Duration>>,

    /// Sets length of the volume fade of song on play/pause.
    #[track_value(pub, pub, eq)]
    #[serde(default = "default::fade_play_pause")]
    #[value_change(Change::FADE_PLAY_PAUSE)]
    fade_play_pause: Wrap<Duration>,

    /// Enable/Disable gapless playback.
    #[track_value(pub, pub, eq)]
    #[serde(default = "default::gapless")]
    #[value_change(Change::GAPLESS)]
    gapless: bool,

    /// Detemines how much uamp seeks with fast forward/rewind
    #[track_value(pub, pub, eq)]
    #[serde(default = "default::seek_jump")]
    seek_jump: Wrap<Duration>,

    /// The port of the server that is used to control uamp.
    #[track_value(pub, pub, eq)]
    #[serde(default = "default::port")]
    #[value_change(Change::PORT)]
    port: u16,

    /// Determines how old must logs be so that they are
    /// automatically deleted.
    #[track_value(pub, pub, eq)]
    #[serde(default = "default::delete_logs_after")]
    delete_logs_after: Wrap<Duration>,

    /// Enable/Disable server that is used to control uamp. Server
    /// is sometimes forced to be enabled and so this has somtimes
    /// no effect.
    #[track_value(pub, pub, eq)]
    #[serde(default = "default::enable_server")]
    #[value_change(Change::ENABLE_SERVER)]
    enable_server: bool,

    /// When jumping to the start of the song, if the command is
    /// issued again within this time, previous song will be played.
    #[track_value(pub, pub, eq)]
    #[serde(default)]
    previous_timeout: Option<Wrap<Duration>>,

    /// If this is true, clients will try to lookup images.
    #[track_value(pub, pub, eq)]
    #[serde(default = "default::client_image_lookup")]
    client_image_lookup: bool,

    /// If enabled, uamp will integrate with the system.
    ///
    /// This is implemented only on linux (unix) using mpris.
    #[track_value(pub, pub, eq)]
    #[serde(default = "default::system_player")]
    #[value_change(Change::SYSTEM_PLAYER)]
    system_player: bool,

    /// If true, uamp will automatically restart when its binary
    /// changes.
    #[track_value(pub, pub, eq)]
    #[serde(default = "default::auto_restart")]
    #[value_change(Change::AUTO_RESTART)]
    auto_restart: bool,

    /// The run type to use by default.
    #[track_value(pub, pub, eq)]
    #[serde(default = "default::default_run_type")]
    default_run_type: RunType,

    // fields that aren't serialized
    #[serde(skip_serializing, default = "default::config_path")]
    pub config_path: Option<PathBuf>,
    #[serde(skip_serializing, default)]
    #[no_update]
    #[no_clone]
    pub force_server: Option<bool>,

    // attributes for the auto field
    #[serde(skip)]
    #[tracker(Cell::set)]
    #[no_update]
    #[no_clone]
    change: Cell<bool>,
}

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

impl Config {
    /// Returns true if some of the saved data changed from the last save.
    pub fn changed(&self) -> bool {
        self.change.get()
    }

    pub fn change(&self) {
        self.set_change(true);
    }

    pub fn reset_change(&self) {
        self.set_change(false);
    }

    /// Creates new config with the given config path
    pub fn new<P>(config_path: Option<P>) -> Self
    where
        P: Into<PathBuf>,
    {
        Config {
            config_path: config_path.map(|p| p.into()),
            ..Default::default()
        }
    }

    pub fn get_cache_cover_path(&self, size: CacheSize) -> PathBuf {
        self.cache_path().join(format!("cover{size}"))
    }

    pub fn should_start_server(&self) -> bool {
        self.force_server.unwrap_or(self.enable_server())
    }

    pub(super) fn update(&mut self, up: serde_json::Value) -> Result<Change> {
        let serde_json::Value::Object(obj) = up else {
            return Err(Error::invalid_value("Expected json object."));
        };

        self.change();

        self.update_from_json_object(obj)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            config_path: default::config_path(),
            change: Cell::new(true),
            ..serde_json::from_str("{}").unwrap()
        }
    }
}
