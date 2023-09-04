pub mod msg;

mod library;
mod load;
mod song;

pub use self::{library::*, msg::Message as LibraryMessage, song::*};

use serde::{Deserialize, Serialize};

/// Id of song in a [`Library`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SongId(usize);

/// Filter for iterating library
pub enum Filter {
    All,
}
