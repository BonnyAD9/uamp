use std::fmt::Display;

use pareg::{ArgError, ArgInto, FromArg};
use serde::{Deserialize, Serialize};

use crate::core::{
    Error, Result,
    library::{Library, SongId},
    player::Player,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Base {
    #[default]
    Library,
    Temporary,
    All,
    None,
    Playlist(usize),
}

impl Base {
    pub fn iter<'a>(
        &self,
        lib: &'a Library,
        player: &'a Player,
    ) -> Result<Box<dyn Iterator<Item = SongId> + 'a>> {
        let res: Box<dyn Iterator<Item = SongId>> = match self {
            Self::Library => Box::new(lib.iter()),
            Self::Temporary => Box::new(lib.iter_tmp()),
            Self::All => Box::new(lib.iter().chain(lib.iter_tmp())),
            Self::None => Box::new(None.into_iter()),
            Self::Playlist(n) => {
                let ps = player.playlist_stack();
                if *n == 0 {
                    Box::new(player.playlist().iter().copied())
                } else if ps.len() >= *n {
                    Box::new(ps[ps.len() - n].iter().copied())
                } else {
                    return Err(Error::invalid_operation()
                        .msg(format!("Invalid playlist index {n}.")));
                }
            }
        };
        Ok(res)
    }
}

impl<'a> FromArg<'a> for Base {
    fn from_arg(arg: &'a str) -> pareg::Result<Self> {
        match arg {
            "lib" | "library" => Ok(Self::Library),
            "tmp" | "temporary" => Ok(Self::Temporary),
            "all" | "_" => Ok(Self::All),
            "none" => Ok(Self::None),
            _ => arg.arg_into().map(Self::Playlist).map_err(|_| {
                ArgError::invalid_value("Invalid query base.", arg).hint(
                    "Expected `lib`, `tmp`, `all`, `none` or playlist \
                        index.",
                )
            }),
        }
    }
}

impl Display for Base {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Library => f.write_str("lib"),
            Self::Temporary => f.write_str("tmp"),
            Self::All => f.write_str("all"),
            Self::None => f.write_str("none"),
            Self::Playlist(p) => write!(f, "{p}"),
        }
    }
}
