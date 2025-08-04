use raplay::Timestamp;
use serde::Serialize;

use crate::core::{
    UampApp,
    server::sub::{Library, Player},
};

#[derive(Debug, Clone, Serialize)]
pub struct SetAll {
    library: Library,
    player: Player,
    position: Option<Timestamp>,
}

impl SetAll {
    pub fn new(app: &mut UampApp) -> Self {
        Self {
            library: Library::new(&mut app.library),
            player: Player::new(&mut app.player),
            position: app.player.timestamp(),
        }
    }
}
