use serde::{Deserialize, Serialize};

use super::{Library, SongId};

/// Defines which field should be primarly ordered by
#[derive(Default, Copy, Clone, Serialize, Deserialize)]
pub enum OrderField {
    /// No ordering
    #[default]
    None,
    /// Order by title
    Title,
    /// Order by track
    Track,
    /// Order by disc and than by track
    Disc,
    /// Order by album, than by disc and than by track
    Album,
    /// Order by artist, than by year, than by album, than by disc and than by
    /// track
    Artist,
    /// Order by year, than by album, than by disc and than by track
    Year,
    /// Order by length
    Length,
    /// Order by genre and than by year
    Genre,
}

/// Defines how to order songs
#[derive(Default, Copy, Clone, Serialize, Deserialize)]
pub struct Order {
    pub field: OrderField,
    /// When true, it will be ordered only by the field
    pub simple: bool,
}

impl Order {
    pub fn _new(field: OrderField, simple: bool) -> Self {
        Self { field, simple }
    }

    pub fn vec(&self, lib: &Library, vec: &mut Vec<SongId>) {
        match self.field {
            OrderField::None => {}
            OrderField::Title => vec.sort_by_key(|s| lib[*s].title()),
            OrderField::Track => vec.sort_by_key(|s| lib[*s].track()),
            OrderField::Disc => {
                if self.simple {
                    vec.sort_by_key(|s| lib[*s].disc());
                } else {
                    vec.sort_by_key(|s| (lib[*s].disc(), lib[*s].track()))
                }
            }
            OrderField::Album => {
                if self.simple {
                    vec.sort_by_key(|s| lib[*s].album())
                } else {
                    vec.sort_by_key(|s| {
                        (lib[*s].album(), lib[*s].disc(), lib[*s].track())
                    })
                }
            }
            OrderField::Artist => {
                if self.simple {
                    vec.sort_by_key(|s| lib[*s].artist());
                } else {
                    vec.sort_by_key(|s| {
                        (
                            lib[*s].artist(),
                            lib[*s].year(),
                            lib[*s].album(),
                            lib[*s].disc(),
                            lib[*s].track(),
                        )
                    })
                }
            }
            OrderField::Year => {
                if self.simple {
                    vec.sort_by_key(|s| lib[*s].year())
                } else {
                    vec.sort_by_key(|s| {
                        (
                            lib[*s].year(),
                            lib[*s].album(),
                            lib[*s].disc(),
                            lib[*s].track(),
                        )
                    })
                }
            }
            OrderField::Length => vec.sort_by_key(|s| lib[*s].length()),
            OrderField::Genre => {
                if self.simple {
                    vec.sort_by_key(|s| lib[*s].genre());
                } else {
                    vec.sort_by_key(|s| (lib[*s].genre(), lib[*s].year()));
                }
            },
        }
    }
}
