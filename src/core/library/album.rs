use std::sync::Arc;

use serde::Serialize;

use crate::core::library::SongId;

#[derive(Debug, Clone, Serialize)]
pub struct Album {
    pub(super) artist: Arc<str>,
    pub(super) name: Arc<str>,
    pub(super) songs: Vec<SongId>,
}

impl Album {
    pub fn new(artist: Arc<str>, name: Arc<str>) -> Self {
        Self {
            artist,
            name,
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
