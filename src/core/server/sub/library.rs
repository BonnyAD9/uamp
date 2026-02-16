use std::sync::Arc;

use serde::Serialize;

use crate::core::library::{self, Albums, Artists, Song};

#[derive(Debug, Serialize, Clone)]
pub struct Library {
    songs: Arc<Vec<Song>>,
    tmp_songs: Arc<Vec<Song>>,
    artists: Arc<Artists>,
    albums: Arc<Albums>,
}

impl Library {
    pub fn new(lib: &mut library::Library) -> Self {
        Self {
            songs: lib.clone_songs().into(),
            tmp_songs: lib.clone_tmp_songs().into(),
            artists: lib.clone_artists().into(),
            albums: lib.clone_albums().into(),
        }
    }
}
