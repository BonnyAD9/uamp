use std::time::Duration;

use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        library::{Library, SongId},
        SongOrder,
    },
    ext::alc_vec::AlcVec,
};

use super::AddPolicy;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Playlist {
    #[serde(default)]
    songs: AlcVec<SongId>,
    #[serde(default)]
    current: usize,
    #[serde(default)]
    play_pos: Option<Duration>,
}

impl Playlist {
    pub fn new<S>(songs: S, mut current: usize) -> Self
    where
        S: Into<AlcVec<SongId>>,
    {
        let songs = songs.into();
        if current > songs.len() {
            current = songs.len().saturating_sub(1);
        }

        Self {
            songs,
            current,
            play_pos: None,
        }
    }

    #[inline(always)]
    pub fn current(&self) -> Option<SongId> {
        (self.current < self.songs.len()).then(|| self.songs[self.current])
    }

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

    pub fn remove_deleted(&mut self, lib: &Library) {
        let cur = self.current();
        self.songs.vec_mut().retain(|s| !lib[s].is_deleted());
        if let Some(cur) = cur {
            self.locate_current(cur)
        }
    }

    pub fn add_songs<I>(&mut self, songs: I, policy: AddPolicy)
    where
        I: IntoIterator<Item = SongId>,
    {
        let i = self.current + 1;
        match policy {
            AddPolicy::End => self.songs.extend(songs),
            AddPolicy::Next => self.songs.splice(i..i, songs),
            AddPolicy::MixIn => self.songs.mix_after(i, songs),
        }
    }

    pub fn sort(&mut self, lib: &Library, simple: bool, order: SongOrder) {
        order.sort(lib, &mut self.songs[..], simple, Some(&mut self.current))
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.songs.len()
    }

    pub fn clone_songs(&mut self) -> AlcVec<SongId> {
        self.songs.clone()
    }

    pub fn get_pos(&self) -> Option<usize> {
        (self.current < self.len()).then_some(self.current)
    }

    pub(super) fn nth_next(&mut self, n: usize) -> Option<SongId> {
        self.current += n;
        if self.current < self.len() {
            self.current()
        } else {
            self.current = 0;
            None
        }
    }

    pub(super) fn nth_prev(&mut self, n: usize) -> Option<SongId> {
        self.jump_to(self.current.saturating_sub(n))
    }

    pub(super) fn jump_to(&mut self, index: usize) -> Option<SongId> {
        (index < self.len()).then(|| {
            self.current = index;
            self.songs[index]
        })
    }

    pub(super) fn set_play_pos(&mut self, play_pos: Duration) {
        self.play_pos = Some(play_pos);
    }

    pub(super) fn pop_play_pos(&mut self) -> Option<Duration> {
        self.play_pos.take()
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

impl Playlist {
    fn locate_current(&mut self, cur: SongId) {
        if let Some(cur) = self.songs.iter().position(|i| *i == cur) {
            self.current = cur;
        }
    }
}
