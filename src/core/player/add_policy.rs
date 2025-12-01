use std::{
    fmt::{Display, Write},
    str::FromStr,
};

use pareg::{ArgError, FromArgStr};
use serde::{Deserialize, Serialize};

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
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" | "none" => Ok(Self::None),
            "e" | "end" => Ok(Self::End),
            "n" | "next" => Ok(Self::Next),
            "m" | "mix" | "mix-in" => Ok(Self::MixIn),
            c => ArgError::failed_to_parse(
                format!("Unknown add policy `{c}`"),
                s,
            )
            .hint("Valid options are: `-`, `e`, `n` or `m`.")
            .err(),
        }
    }
}

impl FromArgStr for AddPolicy {}
