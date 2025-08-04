use std::sync::Arc;

use serde::Serialize;

use crate::core::{
    player::{self, Playback},
    server::sub::Playlist,
};

#[derive(Debug, Clone, Serialize)]
pub struct Player {
    pub playlist: Playlist,
    pub playlist_stack: Arc<Vec<Playlist>>,
    pub volume: f32,
    pub mute: bool,
    pub state: Playback,
}

impl Player {
    pub fn new(pl: &mut player::Player) -> Self {
        pl.get_sub()
    }
}
