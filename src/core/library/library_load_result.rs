use std::{fmt::Debug, mem};

use crate::{
    core::{player::AddPolicy, Error, Result, UampApp},
    env::AppCtrl,
};

use super::{LibraryUpdate, Song, SongId};

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

impl UampApp {
    /// Finishes loading songs started with `start_get_new_songs`.
    ///
    /// This will also start to save the library to json if there is any
    /// change.
    pub(in crate::core) fn finish_library_load(
        &mut self,
        ctrl: &mut AppCtrl,
        res: Option<LibraryLoadResult>,
    ) -> Result<()> {
        let Some(mut res) = res else {
            return Ok(());
        };

        *self.library.songs_mut() = mem::take(&mut res.songs).into();
        if res.removed {
            self.library.update(LibraryUpdate::RemoveData);
        } else {
            self.library.update(LibraryUpdate::NewData);
        }

        self.player.playlist_mut().add_songs(
            (res.first_new..self.library.songs().len())
                .map(SongId)
                .chain(res.sparse_new),
            res.add_policy,
        );

        match self.library.start_to_default_json(
            &self.config,
            ctrl,
            &mut self.player,
        ) {
            Err(Error::InvalidOperation(_)) => Ok(()),
            Err(e) => e
                .prepend("Failed to start library save after library load.")
                .err(),
            _ => Ok(()),
        }
    }
}
