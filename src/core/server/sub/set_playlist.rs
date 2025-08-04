use raplay::Timestamp;
use serde::Serialize;

use crate::core::{
    player::{Playback, Player},
    server::sub::Playlist,
};

#[derive(Debug, Serialize, Clone)]
pub struct SetPlaylist {
    playlist: Playlist,
    timestamp: Option<Timestamp>,
    playback: Playback,
}

impl SetPlaylist {
    pub fn new(pl: &mut Player) -> Self {
        Self {
            playlist: pl.sub_playlist(),
            timestamp: pl.timestamp(),
            playback: pl.playback_state(),
        }
    }
}
