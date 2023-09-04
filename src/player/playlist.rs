use std::{
    ops::{Index, IndexMut},
    slice::{Iter, SliceIndex},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::library::SongId;

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
    pub fn iter<'a>(&'a self) -> Iter<'_, SongId> {
        self[..].iter()
    }

    /// Gets/Creates arc from the playlist
    pub fn as_arc(&self) -> Arc<[SongId]> {
        match self {
            Playlist::Static(a) => a.clone(),
            // copy to arc when this is vector
            Playlist::Dynamic(v) => v[..].into(),
        }
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
        match self {
            Self::Static(_) => {
                self.make_dynamic();
                self.index_mut(index)
            }
            Self::Dynamic(d) => d.index_mut(index),
        }
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
    fn make_dynamic(&mut self) {
        if let Self::Static(s) = self {
            *self = Self::Dynamic(s.as_ref().into())
        }
    }
}
