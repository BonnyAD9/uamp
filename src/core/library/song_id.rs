use serde::{Deserialize, Serialize};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Unique ID of song in a [`Library`].
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct SongId(isize);

impl SongId {
    /// Makes the ID as temporary at the given index.
    #[inline]
    pub(super) fn tmp(idx: usize) -> SongId {
        Self::norm(usize::MAX - idx)
    }

    pub(super) fn norm(idx: usize) -> SongId {
        SongId(idx as isize)
    }

    pub(super) fn as_norm(&self) -> usize {
        self.0 as usize
    }

    /// Interprets the index in this ID as temporary.
    pub(super) fn as_tmp(&self) -> usize {
        usize::MAX - self.as_norm()
    }
}
