pub mod order;

mod add_new_songs;
mod lib;
mod load;
mod song;

pub use self::{
    lib::*,
    load::{LibraryLoadResult, LoadOpts},
    song::*,
};

use pareg::proc::FromArg;
use serde::{Deserialize, Serialize};

/// Id of song in a [`Library`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SongId(usize);

/// Filter for iterating library
#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, PartialEq, FromArg, Default,
)]
pub enum Filter {
    #[default]
    All,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Ord, Eq)]
pub enum LibraryUpdate {
    #[default]
    None = 0,
    /// Some metadata has changed
    Metadata = 1,
    /// There is new data
    NewData = 2,
    /// Some data were removed
    RemoveData = 3,
}
