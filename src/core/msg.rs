use core::fmt::Debug;
use std::sync::Arc;

use crate::{core::UampApp, env::AppCtrl};

use super::{
    config::ConfigMsg, player::PlayerMsg, ControlMsg, DataControlMsg,
    MessageDelegate, PlayMsg,
};

/// Event messages in uamp
#[allow(missing_debug_implementations)]
#[derive(Clone, Debug, Default)]
pub enum Msg {
    /// Play song song at the given index in the playlist.
    PlaySong(PlayMsg),
    /// Some simple messages.
    Control(ControlMsg),
    /// More complicated messages.
    DataControl(DataControlMsg),
    /// Player messges handled by the player.
    Player(PlayerMsg),
    /// Dellegate the message.
    Delegate(Arc<dyn MessageDelegate>),
    /// Message for configuration.
    Config(ConfigMsg),
    #[default]
    None,
}

impl UampApp {
    pub fn msg_event(&mut self, ctrl: &mut AppCtrl, msg: Msg) -> Option<Msg> {
        match msg {
            Msg::PlaySong(msg) => self.play_event(msg),
            Msg::Control(msg) => self.control_event(ctrl, msg),
            Msg::DataControl(msg) => self.data_control_event(ctrl, msg),
            Msg::Player(msg) => self.player_event(msg),
            Msg::Delegate(d) => d.update(self, ctrl),
            Msg::Config(msg) => self.config_event(ctrl, msg),
            Msg::None => None,
        }
    }
}
