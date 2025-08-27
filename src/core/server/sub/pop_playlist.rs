use serde::Serialize;

use crate::core::server::sub::PlaylistJump;

#[derive(Debug, Clone, Serialize)]
pub struct PopPlaylist {
    pop_cnt: usize,
    playlist: PlaylistJump,
}

impl PopPlaylist {
    pub fn new(pop_cnt: usize, playlist: PlaylistJump) -> Self {
        Self { pop_cnt, playlist }
    }
}
