use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use pareg::{ArgError, FromArgStr};
use serde::{Deserialize, Serialize};

use crate::core::player::AddPolicy;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Specifies how new are new songs loaded.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct LoadOpts {
    /// - [`None`] - use value from configuration
    /// - `true` - remove songs from library where the path no longer exists
    /// - `false` - don't remove songs from library where the path no longer
    ///   exists
    pub remove_missing: Option<bool>,
    /// Determines how to add songs to the playlist.
    ///
    /// [`None`] means don't add to playlist.
    pub add_to_playlist: Option<AddPolicy>,
}

impl Display for LoadOpts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::new();

        match self.remove_missing {
            Some(true) => res.push('r'),
            Some(false) => res.push('l'),
            _ => {}
        }

        match self.add_to_playlist {
            Some(AddPolicy::None) => res.push('-'),
            Some(AddPolicy::End) => res.push('e'),
            Some(AddPolicy::Next) => res.push('n'),
            Some(AddPolicy::MixIn) => res.push('m'),
            None => {}
        }

        if res.is_empty() {
            f.write_str("load_songs")
        } else {
            write!(f, "load_songs={res}")
        }
    }
}

impl FromStr for LoadOpts {
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut res = LoadOpts::default();

        let set_rm = |res: &mut LoadOpts, v: bool, i: usize| {
            if res.remove_missing.is_some() {
                ArgError::parse_msg(
                    "Remove missing is set more than once.",
                    s.to_string(),
                )
                .hint("`r` and `l` are mutualy exclusive.")
                .spanned(i..i + 1)
                .err()
            } else {
                res.remove_missing = Some(v);
                Ok(())
            }
        };

        let set_atp = |res: &mut LoadOpts, v: AddPolicy, i: usize| {
            if res.add_to_playlist.is_some() {
                ArgError::parse_msg(
                    "Add policy is set more than once.",
                    s.to_string(),
                )
                .hint("`-`, `e`, `n` and `m` are mutualy exclusive.")
                .spanned(i..i + 1)
                .err()
            } else {
                res.add_to_playlist = Some(v);
                Ok(())
            }
        };

        for (i, c) in s.char_indices() {
            match c {
                'r' => set_rm(&mut res, true, i)?,
                'l' => set_rm(&mut res, false, i)?,
                '-' => set_atp(&mut res, AddPolicy::None, i)?,
                'e' => set_atp(&mut res, AddPolicy::End, i)?,
                'n' => set_atp(&mut res, AddPolicy::Next, i)?,
                'm' => set_atp(&mut res, AddPolicy::MixIn, i)?,
                c => {
                    return ArgError::parse_msg(
                        format!("Invalid option for loading songs: `{c}`"),
                        s.to_string(),
                    )
                    .hint("Valid options are `r`, `l`, `-`, `e`, `n` and `m`")
                    .spanned(i..c.len_utf8())
                    .err();
                }
            }
        }

        Ok(res)
    }
}

impl FromArgStr for LoadOpts {}
