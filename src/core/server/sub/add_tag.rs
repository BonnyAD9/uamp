use std::sync::Arc;

use serde::Serialize;

use crate::core::library::SongId;

#[derive(Debug, Serialize)]
pub struct AddTag {
    hidden: Option<bool>,
    name: Arc<str>,
    songs: Vec<SongId>,
}

impl AddTag {
    pub fn new(
        hidden: Option<bool>,
        name: Arc<str>,
        songs: Vec<SongId>,
    ) -> Self {
        Self {
            hidden,
            name,
            songs,
        }
    }
}
