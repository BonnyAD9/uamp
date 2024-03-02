pub mod cover_image;
pub mod msg;
pub mod order;

mod album;
mod ids;
mod library;
mod load;
mod song;

pub use self::{
    album::*, ids::*, library::*, msg::Message as LibraryMessage, song::*,
};

use serde::{Deserialize, Serialize};

/// Filter for iterating library
pub enum Filter {
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
