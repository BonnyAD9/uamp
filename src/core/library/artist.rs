use std::sync::Arc;

use serde::Serialize;

use crate::core::library::SongId;

pub type ArtistId = Arc<str>;

#[derive(Debug, Clone, Serialize)]
pub struct Artist {
    pub(super) name: Arc<str>,
    pub(super) albums: Vec<Arc<str>>,
    pub(super) singles: Vec<SongId>,
}

impl Artist {
    pub fn new(name: Arc<str>) -> Self {
        Self {
            name,
            albums: vec![],
            singles: vec![],
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn albums(&self) -> &[Arc<str>] {
        &self.albums
    }

    pub fn singles(&self) -> &[SongId] {
        &self.singles
    }
}
