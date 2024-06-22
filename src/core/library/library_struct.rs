use serde_derive::{Deserialize, Serialize};

use std::{
    cell::Cell,
    ops::{Index, IndexMut},
    path::Path,
};

use crate::{core::Result, ext::alc_vec::AlcVec, gen_struct};

use super::{Filter, LibraryUpdate, Song, SongId};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

gen_struct! {
    #[derive(Serialize, Deserialize, Debug)]
    pub Library {
        // Fields passed by reference
        #[doc = "The songs in library. ACCESS VIA GETTER/SETTER whenever"]
        #[doc = "possible."]
        pub(super) songs: AlcVec<Song> { pub(super), pub(super) },
        #[doc = "Temporary songs in the library. They are automatically"]
        #[doc = "removed from library when they are removed from playlist."]
        #[doc = "ACCESS VIA GETTER/SETTER whenever possible."]
        #[serde(default)]
        pub(super) tmp_songs: AlcVec<Song> { pub(super), pub(super) },
        // albums: Vec<SongId> { pri, pri },
        ; // Fields passed by value
        ; // Other fields
        /// invalid song
        #[serde(skip, default = "default_ghost")]
        ghost: Song,
        #[serde(skip)]
        pub(super) lib_update: LibraryUpdate,
        ; // attributes for the auto field
        #[serde(skip)]
    }
}

impl Library {
    /// Creates empty library
    pub fn new() -> Self {
        Library {
            songs: AlcVec::new(),
            tmp_songs: AlcVec::new(),
            lib_update: LibraryUpdate::None,
            change: Cell::new(true),
            ghost: Song::invalid(),
        }
    }

    /// Get clone of the songs in the library.
    pub fn clone_songs(&mut self) -> AlcVec<Song> {
        self.songs.clone()
    }

    /// Change the library update state. Call this when you change some data in
    /// the library - it will eventually propagate the change.
    pub fn update(&mut self, up: LibraryUpdate) {
        if up > self.lib_update {
            self.lib_update = up;
        }
    }

    /// Filters songs in the library
    pub fn filter(&self, filter: Filter) -> AlcVec<SongId> {
        match filter {
            Filter::All => (0..self.songs().len())
                .map(SongId)
                .filter(|s| !self[*s].is_deleted())
                .collect(),
        }
    }

    /// Creates clone of the library. (Works as lazily as possible)
    pub fn clone(&mut self) -> Self {
        Self {
            songs: self.clone_songs(),
            tmp_songs: self.tmp_songs.clone(),
            lib_update: LibraryUpdate::None,
            ghost: self.ghost.clone(),
            change: self.change.clone(),
        }
    }

    /// Add temporary song that will be automatically removed when it is
    /// removed from playlist.
    pub fn add_tmp_song(&mut self, song: Song) -> SongId {
        for (i, s) in self.tmp_songs_mut().iter_mut().enumerate() {
            if s.is_deleted() {
                *s = song;
                return SongId::tmp(i);
            }
        }

        self.tmp_songs_mut().vec_mut().push(song);
        SongId::tmp(self.tmp_songs.len() - 1)
    }

    /// Add temporary song that will be automatically removed when it is
    /// removed from playlist.
    ///
    /// # Errors
    /// - The song fails to load from the given path.
    pub fn add_tmp_path(&mut self, path: impl AsRef<Path>) -> Result<SongId> {
        Ok(self.add_tmp_song(Song::from_path(path)?))
    }

    /// Gets the change value indicating whether the library was changed since
    /// the last save.
    pub(super) fn get_change(&self) -> bool {
        self.change.get()
    }

    /// Sets the change value indicating whether the library was changed since
    /// the last save. Use with caution.
    pub(super) fn set_change(&self, val: bool) {
        self.change.set(val);
    }
}

impl Index<SongId> for Library {
    type Output = Song;
    fn index(&self, index: SongId) -> &Self::Output {
        if index.0 >= self.songs().len() {
            let idx = index.as_tmp();
            if idx >= self.tmp_songs().len() || self.songs()[idx].is_deleted()
            {
                &self.ghost
            } else {
                &self.tmp_songs()[idx]
            }
        } else if self.songs()[index.0].is_deleted() {
            &self.ghost
        } else {
            &self.songs()[index.0]
        }
    }
}

impl IndexMut<SongId> for Library {
    fn index_mut(&mut self, index: SongId) -> &mut Song {
        if index.0 >= self.songs().len() {
            let idx = index.as_tmp();
            if idx >= self.tmp_songs().len() || self.songs()[idx].is_deleted()
            {
                &mut self.ghost
            } else {
                &mut self.tmp_songs_mut()[idx]
            }
        } else if self.songs()[index.0].is_deleted() {
            &mut self.ghost
        } else {
            &mut self.songs_mut()[index.0]
        }
    }
}

impl Index<&SongId> for Library {
    type Output = Song;
    fn index(&self, index: &SongId) -> &Self::Output {
        &self[*index]
    }
}

impl IndexMut<&SongId> for Library {
    fn index_mut(&mut self, index: &SongId) -> &mut Song {
        &mut self[*index]
    }
}

impl Default for Library {
    fn default() -> Self {
        Library::new()
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

fn default_ghost() -> Song {
    Song::invalid()
}
