use core::fmt::Debug;
use std::sync::Arc;

use crate::{core::UampApp, env::AppCtrl};

use super::{
    config::ConfigMsg, player::PlayerMsg, AnyControlMsg, ControlMsg,
    DataControlMsg, MessageDelegate, PlayMsg,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

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
    /// Nothing, just do the usual updates.
    #[default]
    None,
}

impl UampApp {
    /// Handle the message event.
    pub(in crate::core) fn msg_event(
        &mut self,
        ctrl: &mut AppCtrl,
        msg: Msg,
    ) -> Vec<Msg> {
        let mut res = match msg {
            Msg::PlaySong(msg) => self.play_event(msg),
            Msg::Control(msg) => self.control_event(ctrl, msg),
            Msg::DataControl(msg) => self.data_control_event(ctrl, msg),
            Msg::Player(msg) => self.player_event(msg),
            Msg::Delegate(d) => d.update(self, ctrl),
            Msg::Config(msg) => self.config_event(ctrl, msg),
            Msg::None => vec![],
        };

        res.splice(0..0, self.player.get_playlist_action());
        res
    }
}

impl From<PlayMsg> for Msg {
    fn from(value: PlayMsg) -> Self {
        Self::PlaySong(value)
    }
}

impl From<ControlMsg> for Msg {
    fn from(value: ControlMsg) -> Self {
        Self::Control(value)
    }
}

impl From<DataControlMsg> for Msg {
    fn from(value: DataControlMsg) -> Self {
        Self::DataControl(value)
    }
}

impl From<AnyControlMsg> for Msg {
    fn from(value: AnyControlMsg) -> Self {
        match value {
            AnyControlMsg::Control(ctrl) => Self::Control(ctrl),
            AnyControlMsg::Data(data) => Self::DataControl(data),
        }
    }
}

impl From<PlayerMsg> for Msg {
    fn from(value: PlayerMsg) -> Self {
        Self::Player(value)
    }
}

impl From<ConfigMsg> for Msg {
    fn from(value: ConfigMsg) -> Self {
        Msg::Config(value)
    }
}
