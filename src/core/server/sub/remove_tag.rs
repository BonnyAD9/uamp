use std::sync::Arc;

use serde::Serialize;

use crate::core::library::SongId;

#[derive(Debug, Serialize)]
pub struct RemoveTag {
    name: Arc<str>,
    songs: Vec<SongId>,
}

impl RemoveTag {
    pub fn new(name: Arc<str>, songs: Vec<SongId>) -> Self {
        Self { name, songs }
    }
}
