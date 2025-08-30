use std::{collections::HashMap, path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    core::{
        Alias, ControlFunction,
        config::{self, SongPosSave},
    },
    env::{RunType, install},
    ext::Wrap,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Can be modifed over http only from localhost.
    pub library_path: Option<PathBuf>,
    pub player_path: Option<PathBuf>,
    pub cache_path: PathBuf,
    pub search_paths: Vec<PathBuf>,
    pub audio_extensions: Vec<String>,
    pub recursive_search: bool,
    pub server_address: String,
    pub port: u16,
    pub skin: PathBuf,
    pub update_mode: install::UpdateMode,
    pub update_remote: String,
    pub delete_logs_after: Wrap<Duration>,
    pub enable_server: bool,
    pub auto_restart: bool,
    pub web_client_command: Option<String>,

    // Could be modified over http.
    pub control_aliases: HashMap<String, ControlFunction>,
    pub default_playlist_end_action: Option<Alias>,
    pub simple_sorting: bool,
    pub play_on_start: bool,
    pub shuffle_current: bool,
    pub update_library_on_start: bool,
    pub remove_missing_on_load: bool,
    pub volume_jump: f32,
    pub save_playback_pos: SongPosSave,
    pub save_timeout: Option<Wrap<Duration>>,
    pub fade_play_pause: Wrap<Duration>,
    pub gapless: bool,
    pub seek_jump: Wrap<Duration>,
    pub client_image_lookup: bool,
    pub system_player: bool,
    pub default_run_type: RunType,
}

impl Config {
    pub fn new(c: &config::Config) -> Self {
        Self {
            library_path: c.library_path().clone(),
            player_path: c.player_path().clone(),
            cache_path: c.cache_path().clone(),
            search_paths: c.search_paths().clone(),
            audio_extensions: c.audio_extensions().clone(),
            recursive_search: c.recursive_search(),
            server_address: c.server_address().clone(),
            port: c.port(),
            skin: c.skin().clone(),
            update_mode: c.update_mode().clone(),
            update_remote: c.update_remote().clone(),
            delete_logs_after: c.delete_logs_after(),
            enable_server: c.enable_server(),
            auto_restart: c.auto_restart(),
            web_client_command: c.web_client_command().clone(),

            control_aliases: c.control_aliases().clone(),
            default_playlist_end_action: c
                .default_playlist_end_action()
                .clone(),
            simple_sorting: c.simple_sorting(),
            play_on_start: c.play_on_start(),
            shuffle_current: c.shuffle_current(),
            update_library_on_start: c.update_library_on_start(),
            remove_missing_on_load: c.remove_missing_on_load(),
            volume_jump: c.volume_jump(),
            save_playback_pos: c.save_playback_pos(),
            save_timeout: c.save_timeout(),
            fade_play_pause: c.fade_play_pause(),
            gapless: c.gapless(),
            seek_jump: c.seek_jump(),
            client_image_lookup: c.client_image_lookup(),
            system_player: c.system_player(),
            default_run_type: c.default_run_type(),
        }
    }
}
