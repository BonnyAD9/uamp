use std::sync::Arc;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RemoveFromPlaylist {
    ranges: Arc<Vec<[usize; 2]>>,
    playlist: usize,
}

impl RemoveFromPlaylist {
    pub fn new(
        ranges: impl Into<Arc<Vec<[usize; 2]>>>,
        playlist: usize,
    ) -> Self {
        Self {
            ranges: ranges.into(),
            playlist,
        }
    }
}
