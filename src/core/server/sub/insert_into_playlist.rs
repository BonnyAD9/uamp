use std::sync::Arc;

use serde::Serialize;

use crate::core::library::SongId;

#[derive(Debug, Clone, Serialize)]
pub struct InsertIntoPlaylist {
    /// Inserted songs.
    songs: Arc<Vec<SongId>>,
    /// Position within the playlist where to insert.
    position: usize,
    /// Playlist index to which insert.
    playlist: usize,
}

impl InsertIntoPlaylist {
    pub fn new(
        songs: impl Into<Arc<Vec<SongId>>>,
        position: usize,
        playlist: usize,
    ) -> Self {
        InsertIntoPlaylist {
            songs: songs.into(),
            position,
            playlist,
        }
    }
}
