use std::{fmt::Display, str::FromStr};

use pareg::FromArgStr;
use serde::{Deserialize, Serialize};

use super::{ControlMsg, DataControlMsg, Error, Msg};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AnyControlMsg {
    Control(ControlMsg),
    Data(DataControlMsg),
}

impl FromStr for AnyControlMsg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ControlMsg::from_str(s)
            .map(AnyControlMsg::Control)
            .or_else(|_| DataControlMsg::from_str(s).map(AnyControlMsg::Data))
    }
}

impl Display for AnyControlMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnyControlMsg::Control(ctrl) => write!(f, "{ctrl}"),
            AnyControlMsg::Data(data) => write!(f, "{data}"),
        }
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

impl FromArgStr for AnyControlMsg {}
