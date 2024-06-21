use std::fmt::Debug;

use crate::core::player::add_policy::AddPolicy;

use super::{Song, SongId};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Result of library load on another thread
pub struct LibraryLoadResult {
    /// True if any songs were removed from the library.
    pub(super) removed: bool,
    /// The new library contents (all songs not only the new ones)
    pub(super) songs: Vec<Song>,
    /// Determines what to do with the new songs.
    pub(super) add_policy: Option<AddPolicy>,
    /// Index of first new song.
    pub(super) first_new: usize,
    /// New songs with index smaller than [`LibraryLoadResult::first_new`]
    pub(super) sparse_new: Vec<SongId>,
}

impl LibraryLoadResult {
    /// Checks if there is any change in the library.
    pub fn any_change(&self) -> bool {
        self.removed
            || self.first_new != self.songs.len()
            || !self.sparse_new.is_empty()
    }
}

/// Less verbose implementation that doesn't list all the songs
impl Debug for LibraryLoadResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LibraryLoadResult")
            .field("removed", &self.removed)
            .field("songs.len", &self.songs.len())
            .field("add_policy", &self.add_policy)
            .field("first_new", &self.first_new)
            .field("sparse_new.len", &self.sparse_new.len())
            .finish()
    }
}
