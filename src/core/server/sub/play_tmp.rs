use raplay::Timestamp;
use serde::Serialize;

use crate::core::{
    library::{Song, SongId},
    player::{Playback, Player},
    server::sub::Playlist,
};

#[derive(Debug, Clone, Serialize)]
pub struct PlayTmp {
    songs: Vec<(Song, SongId)>,
    playlist: Playlist,
    playback: Playback,
    timestamp: Option<Timestamp>,
}

impl PlayTmp {
    pub fn new(songs: Vec<(Song, SongId)>, player: &mut Player) -> Self {
        Self {
            songs,
            playback: player.playback_state(),
            timestamp: player.timestamp(),
            playlist: player.sub_playlist(),
        }
    }
}
