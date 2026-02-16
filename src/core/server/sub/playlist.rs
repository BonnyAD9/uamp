use std::{sync::Arc, time::Duration};

use serde::Serialize;

use crate::core::{
    Alias,
    library::SongId,
    player::{self, AddPolicy},
};

#[derive(Debug, Clone, Serialize)]
pub struct Playlist {
    songs: Arc<Vec<SongId>>,
    current: Option<usize>,
    play_pos: Option<Duration>,
    on_end: Option<Alias>,
    add_policy: AddPolicy,
}

impl Playlist {
    pub fn new(pl: &mut player::Playlist) -> Self {
        Self {
            songs: pl.clone_songs().into(),
            current: pl.current_idx(),
            play_pos: pl.get_play_pos(),
            on_end: pl.on_end.clone(),
            add_policy: pl.add_policy,
        }
    }
}
