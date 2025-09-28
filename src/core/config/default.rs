use std::{collections::HashMap, path::PathBuf, time::Duration};

use crate::{
    core::{
        Alias, AnyControlMsg, ControlFunction, ControlMsg, DataControlMsg,
        config::{
            DEFAULT_PORT, SongPosSave, default_cache_dir, default_config_dir,
            default_http_client_path, default_plugin_folder,
        },
        player::AddPolicy,
        query::Query,
    },
    env::{RunType, install},
    ext::Wrap,
};

pub fn config_path() -> Option<PathBuf> {
    Some(default_config_dir().join("config.json"))
}

pub fn search_paths() -> Vec<PathBuf> {
    if let Some(dir) = dirs::audio_dir() {
        vec![dir]
    } else {
        vec![PathBuf::from(".")]
    }
}

pub fn library_path() -> Option<PathBuf> {
    Some(default_config_dir().join("library.json"))
}

pub fn player_path() -> Option<PathBuf> {
    Some(default_config_dir().join("player.json"))
}

pub fn cache_path() -> PathBuf {
    default_cache_dir()
}

pub fn audio_extensions() -> Vec<String> {
    vec![
        "flac".to_owned(),
        "mp3".to_owned(),
        "m4a".to_owned(),
        "mp4".to_owned(),
    ]
}

pub fn server_address() -> String {
    "127.0.0.1".to_owned()
}

pub fn control_aliases() -> HashMap<String, ControlFunction> {
    fn end_action(s: &str) -> AnyControlMsg {
        DataControlMsg::SetPlaylistEndAction(Some(Alias::new(s.into()))).into()
    }

    [
        (
            "repeat".into(),
            [
                ControlMsg::PlaylistJump(0).into(),
                ControlMsg::PlayPause(Some(true)).into(),
                end_action("repeat"),
            ]
            .into(),
        ),
        (
            "repeat-once".into(),
            [
                ControlMsg::PlaylistJump(0).into(),
                ControlMsg::PlayPause(Some(true)).into(),
                DataControlMsg::SetPlaylistEndAction(None).into(),
            ]
            .into(),
        ),
        (
            "endless-mix".into(),
            [
                DataControlMsg::SetPlaylist(Query::all_rng()).into(),
                ControlMsg::PlaylistJump(0).into(),
                ControlMsg::PlayPause(Some(true)).into(),
                ControlMsg::SetPlaylistAddPolicy(AddPolicy::MixIn).into(),
                end_action("endless-mix"),
            ]
            .into(),
        ),
        (
            "pcont".into(),
            [
                ControlMsg::PopPlaylist(1).into(),
                ControlMsg::PlayPause(Some(true)).into(),
            ]
            .into(),
        ),
        (
            "palb".into(),
            "[name]:push=a:${name}@+a pp=play spea=pcont"
                .parse()
                .unwrap(),
        ),
    ]
    .into()
}

pub fn update_remote() -> String {
    install::DEFAULT_REMOTE.to_owned()
}

pub fn skin() -> PathBuf {
    default_http_client_path()
}

pub fn plugin_folders() -> Vec<PathBuf> {
    vec![default_plugin_folder()]
}

pub fn shuffle_current() -> bool {
    true
}

pub fn recursive_search() -> bool {
    true
}

pub fn update_library_on_start() -> bool {
    true
}

pub fn remove_missing_on_load() -> bool {
    true
}

pub fn volume_jump() -> f32 {
    0.025
}

pub fn save_playback_pos() -> SongPosSave {
    SongPosSave::OnClose
}

pub fn save_timeout() -> Option<Wrap<Duration>> {
    Some(Wrap(Duration::from_secs(60)))
}

pub fn fade_play_pause() -> Wrap<Duration> {
    Wrap(Duration::from_millis(150))
}

pub fn gapless() -> bool {
    true
}

pub fn seek_jump() -> Wrap<Duration> {
    Wrap(Duration::from_secs(10))
}

pub fn port() -> u16 {
    DEFAULT_PORT
}

pub fn delete_logs_after() -> Wrap<Duration> {
    Wrap(Duration::from_secs(60 * 60 * 24 * 3))
}

pub fn enable_server() -> bool {
    true
}

pub fn client_image_lookup() -> bool {
    true
}

pub fn system_player() -> bool {
    true
}

pub fn auto_restart() -> bool {
    true
}

pub fn default_run_type() -> RunType {
    RunType::WebClient
}
