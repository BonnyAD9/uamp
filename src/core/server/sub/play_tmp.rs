use std::sync::Arc;

use raplay::Timestamp;
use serde::Serialize;

use crate::core::{
    library::{Song, SongId},
    player::{Playback, Player},
    server::sub::Playlist,
};

#[derive(Debug, Clone, Serialize)]
pub struct PlayTmp {
    song: Arc<Song>,
    tmp_id: SongId,
    playlist: Playlist,
    playback: Playback,
    timestamp: Option<Timestamp>,
}

impl PlayTmp {
    pub fn new(song: Arc<Song>, id: SongId, player: &mut Player) -> Self {
        Self {
            song,
            tmp_id: id,
            playback: player.playback_state(),
            timestamp: player.timestamp(),
            playlist: player.sub_playlist(),
        }
    }
}
