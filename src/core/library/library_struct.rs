use itertools::Itertools;
use serde::{Deserialize, Serialize};
use uamp_proc::TrackChange;

use std::{
    cell::Cell,
    collections::HashMap,
    ops::{Index, IndexMut},
    path::Path,
};

use crate::{
    core::{
        Result,
        library::{Album, AlbumId, Artist, ArtistId},
    },
    ext::Alc,
};

use super::{LibraryUpdate, Song, SongId};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

pub type Albums = HashMap<AlbumId, Album>;
pub type Artists = HashMap<ArtistId, Artist>;

#[derive(Serialize, Deserialize, Debug, TrackChange)]
pub struct Library {
    // Fields passed by reference
    /// The songs in library. ACCESS VIA GETTER/SETTER whenever
    /// possible.
    #[track_ref(pub, pub)]
    pub(super) songs: Alc<Vec<Song>>,
    /// Temporary songs in the library. They are automatically
    /// removed from library when they are removed from playlist.
    /// ACCESS VIA GETTER/SETTER whenever possible.
    #[serde(default)]
    #[track_ref(pub, pub)]
    pub(super) tmp_songs: Alc<Vec<Song>>,

    /// Key is (artist, album)
    #[serde(skip, default)]
    pub(super) albums: Alc<Albums>,

    #[serde(skip, default)]
    pub(super) artists: Alc<Artists>,

    // Other fields
    /// invalid song
    #[serde(skip, default = "default_ghost")]
    ghost: Song,
    #[serde(skip)]
    pub(super) lib_update: LibraryUpdate,

    // attributes for the auto field
    #[serde(skip)]
    #[tracker(Cell::set)]
    change: Cell<bool>,
}

impl Library {
    pub(super) fn reset_change(&self) {
        self.set_change(false);
    }

    /// Creates empty library
    pub fn new() -> Self {
        Library {
            songs: Alc::default(),
            tmp_songs: Alc::default(),
            albums: Alc::default(),
            artists: Alc::default(),
            lib_update: LibraryUpdate::None,
            change: Cell::new(true),
            ghost: Song::invalid(),
        }
    }

    pub fn change(&self) {
        self.change.set(true);
    }

    /// Get clone of the songs in the library.
    pub fn clone_songs(&mut self) -> Alc<Vec<Song>> {
        Alc::clone(&mut self.songs)
    }

    pub fn clone_tmp_songs(&mut self) -> Alc<Vec<Song>> {
        Alc::clone(&mut self.tmp_songs)
    }

    pub fn clone_albums(&mut self) -> Alc<Albums> {
        Alc::clone(&mut self.albums)
    }

    pub fn clone_artists(&mut self) -> Alc<Artists> {
        Alc::clone(&mut self.artists)
    }

    /// Change the library update state. Call this when you change some data in
    /// the library - it will eventually propagate the change.
    pub fn update(&mut self, up: LibraryUpdate) {
        if up > self.lib_update {
            self.lib_update = up;
        }
    }

    /*/// Filters songs in the library
    pub fn filter(&self, filter: &Filter) -> AlcVec<SongId> {
        let mut buf = String::new();
        (0..self.songs().len())
            .map(SongId)
            .filter(|s| {
                !self[s].is_deleted() && filter.matches(&self[s], &mut buf)
            })
            .collect()
    }*/

    pub fn iter(&self) -> impl Iterator<Item = SongId> + '_ {
        (0..self.songs().len())
            .map(SongId::norm)
            .filter(|s| !self[s].is_deleted())
    }

    pub fn iter_tmp(&self) -> impl Iterator<Item = SongId> + '_ {
        (0..self.tmp_songs().len())
            .map(SongId::tmp)
            .filter(|s| !self[s].is_deleted())
    }

    /// Creates clone of the library. (Works as lazily as possible)
    pub fn clone(&mut self) -> Self {
        Self {
            songs: self.clone_songs(),
            tmp_songs: Alc::clone(&mut self.tmp_songs),
            albums: Alc::clone(&mut self.albums),
            artists: Alc::clone(&mut self.artists),
            lib_update: LibraryUpdate::None,
            ghost: self.ghost.clone(),
            change: self.change.clone(),
        }
    }

    /// Add temporary song that will be automatically removed when it is
    /// removed from playlist.
    pub fn add_tmp_song(&mut self, song: Song) -> SongId {
        for (i, s) in self.mut_tmp_songs().iter_mut().enumerate() {
            if s.is_deleted() {
                *s = song;
                return SongId::tmp(i);
            }
        }

        self.mut_tmp_songs().push(song);
        SongId::tmp(self.tmp_songs.len() - 1)
    }

    /// Add temporary song that will be automatically removed when it is
    /// removed from playlist.
    ///
    /// # Errors
    /// - The song fails to load from the given path.
    pub fn add_tmp_path(&mut self, path: impl AsRef<Path>) -> Result<SongId> {
        Ok(
            self.add_tmp_song(Song::from_path(path.as_ref()).map_err(
                |e| {
                    e.prepend(format!(
                        "Failed to add tmp song `{}`",
                        path.as_ref().display()
                    ))
                },
            )?),
        )
    }

    pub fn add_tmp_paths(
        &mut self,
        paths: &[impl AsRef<Path>],
    ) -> Result<Vec<SongId>> {
        paths.iter().map(|a| self.add_tmp_path(a)).try_collect()
    }

    /// Checks if song is temporary or not
    pub fn is_tmp(&self, s: SongId) -> bool {
        s.as_norm() >= self.songs().len()
            && s.as_tmp() < self.tmp_songs().len()
    }

    pub fn remove_songs(&mut self, s: impl IntoIterator<Item = SongId>) {
        for s in s {
            self.remove_song_inner(s);
        }
        self.update(LibraryUpdate::RemoveData);
    }

    fn remove_song_inner(&mut self, s: SongId) {
        let s = &mut self[s];
        if !s.is_deleted() {
            s.delete();
        }
    }

    /// Gets the change value indicating whether the library was changed since
    /// the last save.
    pub(super) fn get_change(&self) -> bool {
        self.change.get()
    }
}

impl Index<SongId> for Library {
    type Output = Song;
    fn index(&self, index: SongId) -> &Self::Output {
        if index.as_norm() >= self.songs().len() {
            let idx = index.as_tmp();
            if idx >= self.tmp_songs().len()
                || self.tmp_songs()[idx].is_deleted()
            {
                &self.ghost
            } else {
                &self.tmp_songs()[idx]
            }
        } else if self.songs()[index.as_norm()].is_deleted() {
            &self.ghost
        } else {
            &self.songs()[index.as_norm()]
        }
    }
}

impl IndexMut<SongId> for Library {
    fn index_mut(&mut self, index: SongId) -> &mut Song {
        if index.as_norm() >= self.songs().len() {
            let idx = index.as_tmp();
            if idx >= self.tmp_songs().len() || self.songs()[idx].is_deleted()
            {
                &mut self.ghost
            } else {
                &mut self.mut_tmp_songs()[idx]
            }
        } else if self.songs()[index.as_norm()].is_deleted() {
            &mut self.ghost
        } else {
            &mut self.mut_songs()[index.as_norm()]
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
