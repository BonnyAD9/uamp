use std::{
    fmt::{Display, Write},
    str::FromStr,
};

use pareg::FromArgStr;
use serde::{Deserialize, Serialize};

use crate::core::Error;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Describes how to add songs to a playlist.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum AddPolicy {
    #[default]
    None,
    /// Add songs to the end of the playlist.
    End,
    /// Add songs after the current playing song.
    Next,
    /// Mix the songs randomly after the currently playing song.
    MixIn,
}

impl Display for AddPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => f.write_char('-'),
            Self::End => f.write_char('e'),
            Self::Next => f.write_char('n'),
            Self::MixIn => f.write_char('m'),
        }
    }
}

impl FromStr for AddPolicy {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" | "none" => Ok(Self::None),
            "e" | "end" => Ok(Self::End),
            "n" | "next" => Ok(Self::Next),
            "m" | "mix" | "mix-in" => Ok(Self::MixIn),
            _ => Err(Error::FailedToParse("AddPolicy")),
        }
    }
}

impl FromArgStr for AddPolicy {}
