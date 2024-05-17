use std::{
    mem,
    ops::{Index, IndexMut},
    slice::{Iter, SliceIndex},
    sync::Arc,
};

use itertools::Itertools;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::library::{Library, SongId};

/// A playlist, lazily cloned
pub enum Playlist {
    /// There is static immutable reference to the playlist
    Static(Arc<[SongId]>),
    /// There is owned vector with the playlist
    Dynamic(Vec<SongId>),
}

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

impl Playlist {
    /// Gets the number of items in the playlist
    pub fn len(&self) -> usize {
        match self {
            Self::Static(s) => s.len(),
            Self::Dynamic(d) => d.len(),
        }
    }

    /// Returns iterator over the items
    pub fn iter(&self) -> Iter<'_, SongId> {
        self[..].iter()
    }

    /// Gets/Creates arc from the playlist. This will create copy if playlist
    /// is dynamic.
    pub fn _to_arc(&self) -> Arc<[SongId]> {
        match self {
            Playlist::Static(a) => a.clone(),
            // copy to arc when this is vector
            Playlist::Dynamic(v) => v[..].into(),
        }
    }

    /// Gets/Creates arc from the playlist. This will not copy the playlist,
    /// but it will transform it into a static playlist.
    pub fn _as_arc(&mut self) -> Arc<[SongId]> {
        self._make_static().clone()
    }

    pub fn remove_deleted(&mut self, lib: &Library) {
        match self {
            Playlist::Static(a) => {
                *self = Playlist::Dynamic(
                    a.iter()
                        .copied()
                        .filter(|s| !lib[*s].is_deleted())
                        .collect_vec(),
                )
            }
            Playlist::Dynamic(v) => v.retain(|s| !lib[*s].is_deleted()),
        }
    }

    pub fn mix_in<I>(&mut self, after: usize, iter: I)
    where
        I: IntoIterator<Item = SongId>,
    {
        let mut it = iter.into_iter();
        let i = it.next();
        if i.is_none() {
            return;
        }

        let v = self.make_dynamic();
        let mut rng = rand::thread_rng();
        for s in i.into_iter().chain(it) {
            let idx = rng.gen_range(after + 1..=v.len());
            let o = v[idx];
            v[idx] = s;
            v.push(o);
        }
    }

    pub fn insert_at<I>(&mut self, idx: usize, iter: I)
    where
        I: IntoIterator<Item = SongId>,
    {
        let mut it = iter.into_iter();
        let i = it.next();
        if i.is_none() {
            return;
        }

        let v = self.make_dynamic();
        v.splice(idx..idx, i.into_iter().chain(it));
    }
}

impl Default for Playlist {
    fn default() -> Self {
        [][..].into()
    }
}

impl<T: SliceIndex<[SongId]>> Index<T> for Playlist {
    type Output = T::Output;

    fn index(&self, index: T) -> &Self::Output {
        match self {
            Self::Static(s) => s.index(index),
            Self::Dynamic(d) => d.index(index),
        }
    }
}

impl<T: SliceIndex<[SongId]>> IndexMut<T> for Playlist {
    #[inline]
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        self.make_dynamic().index_mut(index)
    }
}

impl From<&[SongId]> for Playlist {
    fn from(value: &[SongId]) -> Self {
        Self::Dynamic(value.into())
    }
}

impl From<Vec<SongId>> for Playlist {
    fn from(value: Vec<SongId>) -> Self {
        Self::Dynamic(value)
    }
}

impl From<Arc<[SongId]>> for Playlist {
    fn from(value: Arc<[SongId]>) -> Self {
        Self::Static(value)
    }
}

impl Extend<SongId> for Playlist {
    fn extend<T: IntoIterator<Item = SongId>>(&mut self, iter: T) {
        let mut it = iter.into_iter();
        let i = it.next();
        if i.is_some() {
            self.make_dynamic().extend(i.into_iter().chain(it))
        }
    }
}

impl<'a> Extend<&'a SongId> for Playlist {
    fn extend<T: IntoIterator<Item = &'a SongId>>(&mut self, iter: T) {
        let mut it = iter.into_iter();
        let i = it.next();
        if i.is_some() {
            self.make_dynamic().extend(i.into_iter().chain(it))
        }
    }
}

impl Serialize for Playlist {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Static(s) => s.as_ref().serialize(serializer),
            Self::Dynamic(v) => v.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Playlist {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::Dynamic(Vec::deserialize(deserializer)?))
    }

    fn deserialize_in_place<D>(
        deserializer: D,
        place: &mut Self,
    ) -> std::result::Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match place {
            Playlist::Static(_) => {
                *place = Self::Dynamic(Vec::deserialize(deserializer)?);
                Ok(())
            }
            Playlist::Dynamic(v) => Vec::deserialize_in_place(deserializer, v),
        }
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl Playlist {
    /// Clones the playlist and creates owned vector
    #[inline]
    fn make_dynamic(&mut self) -> &mut Vec<SongId> {
        match self {
            Self::Dynamic(d) => d,
            Self::Static(s) => {
                *self = Self::Dynamic(s.as_ref().into());
                let Self::Dynamic(d) = self else { panic!() };
                d
            }
        }
    }

    #[inline]
    fn _make_static(&mut self) -> &Arc<[SongId]> {
        match self {
            Self::Dynamic(d) => {
                *self = Self::Static(mem::take(d).into());
                let Self::Static(s) = self else {
                    panic!();
                };
                s
            }
            Self::Static(s) => s,
        }
    }
}
