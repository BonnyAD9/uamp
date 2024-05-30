use std::{fmt::Debug, str::FromStr};

use pareg::FromArgStr;
use serde::{Deserialize, Serialize};

use crate::{core::extensions::ParseError, player::add_policy::AddPolicy};

use super::{Song, SongId};

/// Result of library load on another thread
pub struct LibraryLoadResult {
    pub(super) removed: bool,
    pub(super) songs: Vec<Song>,
    pub(super) add_policy: Option<AddPolicy>,
    pub(super) first_new: usize,
    pub(super) sparse_new: Vec<SongId>,
}

impl LibraryLoadResult {
    pub fn any_change(&self) -> bool {
        self.removed
            || self.first_new != self.songs.len()
            || !self.sparse_new.is_empty()
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct LoadOpts {
    pub remove_missing: Option<bool>,
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
            Some(AddPolicy::End) => res.push('e'),
            Some(AddPolicy::Next) => res.push('n'),
            Some(AddPolicy::MixIn) => res.push('m'),
            _ => {}
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
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut res = LoadOpts::default();

        fn set_rm(res: &mut LoadOpts, v: bool) -> Result<(), ParseError> {
            if res.remove_missing.is_some() {
                Err(ParseError::FailedToParse("LoadOpts"))
            } else {
                res.remove_missing = Some(v);
                Ok(())
            }
        }

        fn set_atp(
            res: &mut LoadOpts,
            v: AddPolicy,
        ) -> Result<(), ParseError> {
            if res.add_to_playlist.is_some() {
                Err(ParseError::FailedToParse("LoadOpts"))
            } else {
                res.add_to_playlist = Some(v);
                Ok(())
            }
        }

        for c in s.chars() {
            match c {
                'r' => set_rm(&mut res, true)?,
                'l' => set_rm(&mut res, false)?,
                'e' => set_atp(&mut res, AddPolicy::End)?,
                'n' => set_atp(&mut res, AddPolicy::Next)?,
                'm' => set_atp(&mut res, AddPolicy::MixIn)?,
                _ => {
                    return Err(ParseError::FailedToParse("LoadOpts"));
                }
            }
        }

        Ok(res)
    }
}

impl FromArgStr for LoadOpts {}

impl Debug for LibraryLoadResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LibraryLoadResult")
            .field("removed", &self.removed)
            .field("add_policy", &self.add_policy)
            .field("first_new", &self.first_new)
            .finish()
    }
}
