use std::{ops::Index, slice::SliceIndex, time::Duration};

use log::error;
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        library::{Library, SongId},
        query::SongOrder,
        Alias,
    },
    ext::AlcVec,
};

use super::AddPolicy;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Song playlist.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Playlist {
    #[serde(default)]
    songs: AlcVec<SongId>,
    #[serde(default)]
    current: usize,
    #[serde(default)]
    play_pos: Option<Duration>,
    /// Alias to run when the playlist ends.
    #[serde(default)]
    pub on_end: Option<Alias>,
    /// How songs should be added to the playlist.
    #[serde(default)]
    pub add_policy: Option<AddPolicy>,
}

impl Playlist {
    /// Creates new playlist from the given songs and current song index.
    ///
    /// If the index is invalid, it is set to the last song.
    pub fn new(songs: impl Into<AlcVec<SongId>>, mut current: usize) -> Self {
        let songs = songs.into();
        if current > songs.len() {
            current = songs.len().saturating_sub(1);
        }

        Self {
            songs,
            current,
            play_pos: None,
            on_end: None,
            add_policy: None,
        }
    }

    /// Gets the song id of the current song.
    #[inline(always)]
    pub fn current(&self) -> Option<SongId> {
        self.current_idx().map(|c| self.songs[c])
    }

    /// Gets the index of the current song in the playlist
    #[inline(always)]
    pub fn current_idx(&self) -> Option<usize> {
        (self.current < self.songs.len()).then_some(self.current)
    }

    /// Shuffles the playlist.
    ///
    /// - `shuffle_current`: when `false`, the current song will be moved to
    ///   the first position.
    pub fn shuffle(&mut self, shuffle_current: bool) {
        let id = self.current();
        self.songs[..].shuffle(&mut thread_rng());
        // find the currently playing in the shuffled playlist
        if let Some(id) = id {
            self.locate_current(id);
            if !shuffle_current {
                self.songs[..].swap(self.current, 0);
                self.current = 0;
            }
        }
    }

    /// Removes all deleted songs.
    pub fn remove_deleted(&mut self, lib: &Library) {
        let cur = self.current();
        let old_len = self.len();
        self.songs.vec_mut().retain(|s| !lib[s].is_deleted());
        if let Some(cur) = cur {
            let len_diff = old_len - self.len();
            self.locate_current_h(cur, self.current - len_diff, len_diff + 1);
        }
    }

    /// Add songs to the playlist.
    ///
    /// - `songs`: Iterator over songs to add.
    /// - `policy`: Where in the playlist the songs should be added.
    pub fn add_songs<I>(&mut self, songs: I, policy: Option<AddPolicy>)
    where
        I: IntoIterator<Item = SongId>,
    {
        let Some(policy) = policy.or(self.add_policy) else {
            return;
        };

        let i = self.current + 1;
        match policy {
            AddPolicy::None => {}
            AddPolicy::End => self.songs.extend(songs),
            AddPolicy::Next => self.songs.splice(i..i, songs),
            AddPolicy::MixIn => self.songs.mix_after(i, songs),
        }
    }

    /// Sorts the songs according to the song order.
    pub fn sort(&mut self, lib: &Library, simple: bool, order: SongOrder) {
        order.sort(lib, &mut self.songs[..], simple, Some(&mut self.current))
    }

    /// Gets the length of the playlist.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.songs.len()
    }

    /// Creates lazy copy of the songs in the playlist.
    pub fn clone_songs(&mut self) -> AlcVec<SongId> {
        self.songs.clone()
    }

    /// Gets the current song index in the playlist.
    pub fn get_pos(&self) -> Option<usize> {
        (self.current < self.len()).then_some(self.current)
    }

    /// Gets iterator over all of the songs in the playlist.
    pub fn iter(&self) -> std::slice::Iter<'_, SongId> {
        self.songs.iter()
    }

    /// Gets mutable iterator over all of the songs in the playlist
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, SongId> {
        self.songs.iter_mut()
    }

    /// Adds the songs to the playlist after the currently playing song.
    pub fn play_next(&mut self, iter: impl IntoIterator<Item = SongId>) {
        let c = self.current + 1;
        self.songs.splice(c..c, iter)
    }

    /// Moves current to the nth next song and returns its id.
    ///
    /// If the nth next song is outside of the playlist, jump to the first song
    /// in the playlist.
    ///
    /// # Returns
    /// The song id if the nth next song was in the playlist.
    pub(super) fn nth_next(&mut self, n: usize) -> Option<SongId> {
        self.current += n;
        if self.current < self.len() {
            self.current()
        } else {
            self.current = 0;
            None
        }
    }

    /// Moves current to the nth previous song and returns its id.
    ///
    /// If the position is outside of the playlist, move to the first song in
    /// the playlist.
    ///
    /// # Returns
    /// [`None`] if the playlist is empty.
    pub(super) fn nth_prev(&mut self, n: usize) -> Option<SongId> {
        self.jump_to(self.current.saturating_sub(n))
    }

    /// Jumps to the given index and returns song id at the new position.
    ///
    /// The index is clamped to the proper playlist range.
    ///
    /// # Returns
    /// [`None`] if the playlist is empty.
    pub(super) fn jump_to(&mut self, index: usize) -> Option<SongId> {
        self.current = index.clamp(0, self.len().saturating_sub(1));
        self.current()
    }

    /// Stores the position within the current song in the playlist.
    pub(super) fn set_play_pos(&mut self, play_pos: Duration) {
        self.play_pos = Some(play_pos);
    }

    /// Gets the position within song in the current playlists and unsets it.
    pub(super) fn pop_play_pos(&mut self) -> Option<Duration> {
        self.play_pos.take()
    }

    /// Removes the current song from the playlist, moves the position to the
    /// next song and returns the removed song id.
    pub(super) fn pop_current(&mut self) -> Option<SongId> {
        self.current_idx().map(|c| {
            let s = self.songs[c];
            self.songs.vec_mut().remove(c);
            s
        })
    }

    /// Insert `other` at the current position. If current position is [`None`]
    /// it is inserted at the end. Retain the now playing from the `other`
    /// playlist.
    pub(super) fn flatten(&mut self, other: Playlist) {
        let pos = self.current_idx().unwrap_or(self.songs.len());
        let cur = other.current_idx().map(|a| a + pos);
        self.songs.splice(pos..pos, other.iter().copied());
        self.current = cur.unwrap_or(self.songs.len());
        self.play_pos = other.play_pos;
    }
}

impl<V> From<V> for Playlist
where
    V: Into<AlcVec<SongId>>,
{
    fn from(value: V) -> Self {
        Self::new(value, 0)
    }
}

impl<'a> IntoIterator for &'a mut Playlist {
    type Item = &'a mut SongId;

    type IntoIter = std::slice::IterMut<'a, SongId>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a> IntoIterator for &'a Playlist {
    type Item = &'a SongId;

    type IntoIter = std::slice::Iter<'a, SongId>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Extend<SongId> for Playlist {
    fn extend<T: IntoIterator<Item = SongId>>(&mut self, iter: T) {
        self.songs.extend(iter)
    }
}

impl<I: SliceIndex<[SongId]>> Index<I> for Playlist {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.songs.index(index)
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl Playlist {
    fn locate_current(&mut self, cur: SongId) {
        if let Some(cur) = self.songs.iter().position(|i| *i == cur) {
            self.current = cur;
        } else {
            error!("Failed to locate current song?, this is a bug!");
        }
    }

    fn locate_current_h(&mut self, cur: SongId, start: usize, count: usize) {
        let songs = &self.songs[start..start + count];
        if let Some(cur) = songs.iter().position(|i| *i == cur) {
            self.current = cur + start;
        } else {
            error!("Failed to locate current song?, this is a bug! (h).");
            self.locate_current(cur);
        }
    }
}
