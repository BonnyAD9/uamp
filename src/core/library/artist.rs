use std::sync::Arc;

use crate::core::library::SongId;

#[derive(Debug, Clone)]
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
