use std::sync::Arc;

use super::{Library, SongId};
use crate::core::{err::Result, Error};

pub struct Album {
    // sorted by disc and track
    name: Arc<str>,
    songs: Vec<SongId>,
}

impl Album {
    pub fn name<'a>(&self) -> &str {
        &self.name
    }

    pub fn artist<'a>(&self, lib: &'a Library) -> &'a str {
        lib[self.songs[0]].artist()
    }

    pub fn songs(&self) -> &[SongId] {
        &self.songs[..]
    }

    pub fn count(&self) -> usize {
        self.songs.len()
    }

    pub fn year(&self, lib: &Library) -> i32 {
        lib[self.songs[0]].year()
    }

    pub fn genre<'a>(&self, lib: &'a Library) -> &'a str {
        lib[self.songs[0]].genre()
    }

    pub fn is_deleted(&self) -> bool {
        self.count() == 0
    }

    pub(super) fn sort(&mut self, lib: &Library) {
        self.songs
            .sort_by_key(|s| (lib[*s].disc(), lib[*s].track()));
    }

    pub(super) fn remove_song(&mut self, s: SongId) -> Result<()> {
        if let Some(p) = self.songs.iter().position(|i| s == *i) {
            self.songs.remove(p);
            Ok(())
        } else {
            Err(Error::InvalidOperation(
                "Cannot remove song from album, it is not part of the album",
            ))
        }
    }
}
