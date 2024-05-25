use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::player::add_policy::AddPolicy;

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

impl Debug for LibraryLoadResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LibraryLoadResult")
            .field("removed", &self.removed)
            .field("add_policy", &self.add_policy)
            .field("first_new", &self.first_new)
            .finish()
    }
}
