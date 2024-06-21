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
pub struct SongId(pub(super) usize);

impl SongId {
    /// Makes the ID as temporary at the given index.
    #[inline]
    pub(super) fn tmp(idx: usize) -> SongId {
        SongId(usize::MAX - idx)
    }

    /// Interprets the index in this ID as temporary.
    pub(super) fn as_tmp(&self) -> usize {
        usize::MAX - self.0
    }
}
