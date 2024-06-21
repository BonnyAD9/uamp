use std::fmt::Display;

use pareg::proc::FromArg;
use serde::{Deserialize, Serialize};

/// Filter for iterating library song ids.
#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, PartialEq, FromArg, Default,
)]
pub enum Filter {
    #[default]
    All,
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => f.write_str("all"),
        }
    }
}
