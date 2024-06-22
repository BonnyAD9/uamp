use std::{fmt::Display, str::FromStr};

use pareg::{key_val_arg, ArgError, FromArgStr};
use serde::{Deserialize, Serialize};

use crate::{env::AppCtrl, starts};

use super::{Error, Msg, UampApp};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Messages that can be safely send across threads, but not necesarily esily
/// copied.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DataControlMsg {
    /// Invoke alias.
    Alias(String),
}

impl UampApp {
    /// Handles events for [`DataControlMsg`]
    pub fn data_control_event(
        &mut self,
        ctrl: &mut AppCtrl,
        msg: DataControlMsg,
    ) -> Option<Msg> {
        match msg {
            DataControlMsg::Alias(name) => {
                for m in self.config.control_aliases().get(&name)?.clone() {
                    self.update(ctrl, m.into())
                }

                None
            }
        }
    }
}

impl FromStr for DataControlMsg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            v if starts!(v, "al" | "alias") => {
                Ok(DataControlMsg::Alias(key_val_arg::<&str, _>(v, '=')?.1))
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
        }
    }
}

impl FromArgStr for DataControlMsg {}
