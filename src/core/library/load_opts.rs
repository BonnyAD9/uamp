use std::{fmt::Debug, str::FromStr};

use pareg::FromArgStr;
use serde::{Deserialize, Serialize};

use crate::core::{player::AddPolicy, Error};

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

impl ToString for LoadOpts {
    fn to_string(&self) -> String {
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
            "load-songs".into()
        } else {
            res.insert_str(0, "load_songs=");
            res
        }
    }
}

impl FromStr for LoadOpts {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut res = LoadOpts::default();

        fn set_rm(res: &mut LoadOpts, v: bool) -> Result<(), Error> {
            if res.remove_missing.is_some() {
                Err(Error::FailedToParse("LoadOpts"))
            } else {
                res.remove_missing = Some(v);
                Ok(())
            }
        }

        fn set_atp(res: &mut LoadOpts, v: AddPolicy) -> Result<(), Error> {
            if res.add_to_playlist.is_some() {
                Err(Error::FailedToParse("LoadOpts"))
            } else {
                res.add_to_playlist = Some(v);
                Ok(())
            }
        }

        for c in s.chars() {
            match c {
                'r' => set_rm(&mut res, true)?,
                'l' => set_rm(&mut res, false)?,
                '-' => set_atp(&mut res, AddPolicy::None)?,
                'e' => set_atp(&mut res, AddPolicy::End)?,
                'n' => set_atp(&mut res, AddPolicy::Next)?,
                'm' => set_atp(&mut res, AddPolicy::MixIn)?,
                _ => {
                    return Err(Error::FailedToParse("LoadOpts"));
                }
            }
        }

        Ok(res)
    }
}

impl FromArgStr for LoadOpts {}