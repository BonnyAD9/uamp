use raplay::Timestamp;
use serde::Serialize;

use crate::core::player::{Playback, Player};

#[derive(Debug, Clone, Serialize)]
pub struct PlaylistJump {
    position: Option<usize>,
    playback: Playback,
    timestamp: Option<Timestamp>,
}

impl PlaylistJump {
    pub fn new(pl: &Player) -> Self {
        Self {
            position: pl.playlist().current_idx(),
            playback: pl.playback_state(),
            timestamp: pl.timestamp(),
        }
    }
}
