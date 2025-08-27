use serde::Serialize;

use crate::core::server::sub::SetPlaylist;

#[derive(Debug, Clone, Serialize)]
pub struct PopSetPlaylist {
    // Number of playlists to pop. 0 means pop all.
    pop_cnt: usize,
    playlist: SetPlaylist,
}

impl PopSetPlaylist {
    pub fn new(pop_cnt: usize, playlist: SetPlaylist) -> Self {
        Self { pop_cnt, playlist }
    }
}
