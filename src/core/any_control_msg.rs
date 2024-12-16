use std::{fmt::Display, str::FromStr};

use pareg::{ArgError, FromArgStr};
use serde::{Deserialize, Serialize};

use super::{ControlMsg, DataControlMsg};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Either [`ControlMsg`] or [`DataControlMsg`]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AnyControlMsg {
    /// [`ControlMsg`]
    Control(ControlMsg),
    /// [`DataControlMsg`]
    Data(DataControlMsg),
}

impl FromStr for AnyControlMsg {
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match ControlMsg::from_str(s) {
            Ok(r) => Ok(AnyControlMsg::Control(r)),
            Err(ArgError::UnknownArgument(_)) => {
                DataControlMsg::from_str(s).map(AnyControlMsg::Data)
            }
            Err(e) => Err(e),
        }
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

impl FromArgStr for AnyControlMsg {}

impl From<ControlMsg> for AnyControlMsg {
    fn from(value: ControlMsg) -> Self {
        Self::Control(value)
    }
}

impl From<DataControlMsg> for AnyControlMsg {
    fn from(value: DataControlMsg) -> Self {
        Self::Data(value)
    }
}
