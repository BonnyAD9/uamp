use std::sync::Arc;

use serde::Serialize;

use crate::core::library::SongId;

/// (Artist, Name)
pub type AlbumId = (Arc<str>, Arc<str>);

#[derive(Debug, Clone, Serialize)]
pub struct Album {
    pub(super) name: Arc<str>,
    pub(super) artist: Arc<str>,
    pub(super) songs: Vec<SongId>,
}

impl Album {
    pub fn new(name: Arc<str>, artist: Arc<str>) -> Self {
        Self {
            name,
            artist,
            songs: vec![],
        }
    }

    pub fn songs(&self) -> &[SongId] {
        &self.songs
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn artist(&self) -> &str {
        &self.artist
    }
}
