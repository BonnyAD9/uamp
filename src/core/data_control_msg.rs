use std::{fmt::Display, str::FromStr};

use pareg::{
    has_any_key, mval_arg, starts_any, val_arg, ArgError, FromArgStr,
};
use serde::{Deserialize, Serialize};

use crate::env::AppCtrl;

use super::{control_msg_vec::ControlMsgVec, Error, Msg, UampApp};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Messages that can be safely send across threads, but not necesarily esily
/// copied.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DataControlMsg {
    /// Invoke alias.
    Alias(String),
    /// Sets the current playlist end action.
    SetPlaylistEndAction(Option<String>),
}

impl UampApp {
    /// Handles events for [`DataControlMsg`]
    pub(in crate::core) fn data_control_event(
        &mut self,
        _ctrl: &mut AppCtrl,
        msg: DataControlMsg,
    ) -> Vec<Msg> {
        match msg {
            DataControlMsg::Alias(name) => {
                return self
                    .config
                    .control_aliases()
                    .get(&name)
                    .map(ControlMsgVec::get_msg_vec)
                    .unwrap_or_default()
            }
            DataControlMsg::SetPlaylistEndAction(act) => {
                self.player.playlist_mut().on_end = act;
            }
        }

        vec![]
    }
}

impl FromStr for DataControlMsg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            v if starts_any!(v, "al", "alias") => {
                Ok(DataControlMsg::Alias(val_arg(v, '=')?))
            }
            v if has_any_key!(
                v,
                '=',
                "spea",
                "pl-end",
                "playlist-end",
                "playlist-end-action"
            ) =>
            {
                Ok(DataControlMsg::SetPlaylistEndAction(mval_arg(v, '=')?))
            }
            v => Err(Error::ArgParse(ArgError::UnknownArgument(
                v.to_owned().into(),
            ))),
        }
    }
}

impl Display for DataControlMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataControlMsg::Alias(alias) => write!(f, "al={alias}"),
            DataControlMsg::SetPlaylistEndAction(None) => write!(f, "spea"),
            DataControlMsg::SetPlaylistEndAction(Some(act)) => {
                write!(f, "spea={act}")
            }
        }
    }
}

impl FromArgStr for DataControlMsg {}