use std::sync::Arc;

use itertools::Itertools;
use serde::Serialize;

use crate::core::{
    player::{self, Playback},
    server::sub::Playlist,
};

#[derive(Debug, Clone, Serialize)]
pub struct Player {
    playlist: Playlist,
    playlist_stack: Arc<Vec<Playlist>>,
    volume: f32,
    mute: bool,
    state: Playback,
}

impl Player {
    pub fn new(pl: &mut player::Player) -> Self {
        // TODO: dont mutate player (dont make it save)
        Self {
            playlist: Playlist::new(pl.playlist_mut()),
            playlist_stack: pl
                .playlist_stack_mut()
                .iter_mut()
                .map(Playlist::new)
                .collect_vec()
                .into(),
            volume: pl.volume(),
            mute: pl.mute(),
            state: pl.playback_state(),
        }
    }
}
