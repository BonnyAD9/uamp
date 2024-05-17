use std::{thread::JoinHandle, time::Instant};

use serde::{Deserialize, Serialize};

use crate::player::add_policy::AddPolicy;

use super::{Song, SongId};

/// Contains metadata about library load on another thread
pub struct LibraryLoad {
    pub handle: JoinHandle<Option<LibraryLoadResult>>,
    pub time_started: Instant,
}

/// Result of library load on another thread
pub struct LibraryLoadResult {
    pub removed: bool,
    pub songs: Vec<Song>,
    pub add_policy: Option<AddPolicy>,
    pub first_new: usize,
    pub sparse_new: Vec<SongId>,
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
