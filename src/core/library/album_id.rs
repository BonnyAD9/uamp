use std::sync::Arc;

use serde::Serialize;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct AlbumId {
    pub artist: Arc<str>,
    pub name: Arc<str>,
}

impl AlbumId {
    pub fn new(
        artist: impl Into<Arc<str>>,
        name: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            artist: artist.into(),
            name: name.into(),
        }
    }
}

impl Serialize for AlbumId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{}\t{}", self.artist, self.name).serialize(serializer)
    }
}
