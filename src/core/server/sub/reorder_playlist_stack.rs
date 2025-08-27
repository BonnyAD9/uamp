use std::sync::Arc;

use serde::Serialize;

use crate::core::server::sub::PlaylistJump;

#[derive(Debug, Clone, Serialize)]
pub struct ReorderPlaylistStack {
    order: Arc<Vec<usize>>,
    // Current position in the new top of the stack
    position: PlaylistJump,
}

impl ReorderPlaylistStack {
    pub fn new(order: Arc<Vec<usize>>, position: PlaylistJump) -> Self {
        Self { order, position }
    }
}
