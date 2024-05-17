pub mod msg;
pub mod order;

mod lib;
mod load;
mod song;

use crate::core::extensions::{ParseError, Parses};

pub use self::{
    lib::*, load::LoadOpts, msg::Message as LibraryMessage, song::*,
};

use serde::{Deserialize, Serialize};

/// Id of song in a [`Library`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SongId(usize);

/// Filter for iterating library
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Filter {
    All,
}

impl Parses<Filter> for str {
    type Err = ParseError;

    fn get_value(&self) -> Result<Filter, Self::Err> {
        match self {
            "all" => Ok(Filter::All),
            _ => Err(ParseError::FailedToParse("Filter")),
        }
    }
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
