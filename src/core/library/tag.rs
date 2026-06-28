use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::{config::AutoTag, library::SongId};

pub type TagId = Arc<str>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub name: Arc<str>,
    pub songs: Vec<SongId>,
    pub hidden: bool,
}

impl Tag {
    pub fn new(name: impl Into<Arc<str>>, hidden: bool) -> Self {
        Self {
            name: name.into(),
            hidden,
            songs: vec![],
        }
    }

    pub fn visible(name: impl Into<Arc<str>>) -> Self {
        Self::new(name, false)
    }
}

impl From<AutoTag> for Tag {
    fn from(value: AutoTag) -> Self {
        Self::new(value.name, value.hidden)
    }
}
