use raplay::Timestamp;
use serde::Serialize;

use crate::core::{
    UampApp,
    server::{
        sub::{Config, Library, Player},
        sub_msg,
    },
};

#[derive(Debug, Clone, Serialize)]
pub struct SetAll {
    sse: &'static [&'static str],
    library: Library,
    player: Player,
    position: Option<Timestamp>,
    config: Config,
}

impl SetAll {
    pub fn new(app: &mut UampApp) -> Self {
        Self {
            sse: sub_msg::EVENTS,
            library: Library::new(&mut app.library),
            player: Player::new(&mut app.player),
            position: app.player.timestamp(),
            config: Config::new(&app.config),
        }
    }
}
