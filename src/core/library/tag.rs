use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::library::SongId;

pub type TagId = Arc<str>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub name: Arc<str>,
    pub songs: Vec<SongId>,
    pub hidden: bool,
}

impl Tag {
    pub fn new(name: impl Into<Arc<str>>) -> Self {
        Self {
            name: name.into(),
            songs: vec![],
            hidden: false,
        }
    }
}
