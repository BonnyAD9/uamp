use std::cmp::Reverse;

use serde::{Deserialize, Serialize};

use super::{Library, SongId};

/// Defines which field should be primarly ordered by
#[derive(
    Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug,
)]
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
#[derive(
    Default, Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq,
)]
pub struct Order {
    pub field: OrderField,
    /// When true, it will be ordered only by the field
    pub simple: bool,
    pub reverse: bool,
}

impl Order {
    pub fn _new(field: OrderField, simple: bool) -> Self {
        Self {
            field,
            simple,
            reverse: false,
        }
    }

    pub fn _set_rev(mut self, rev: bool) -> Self {
        self.reverse = rev;
        self
    }

    pub fn _vec(&self, lib: &Library, vec: &mut [SongId]) {
        match self.field {
            OrderField::None => {}
            OrderField::Title => {
                Self::_sort(vec, self.reverse, |s| lib[*s].title())
            }
            OrderField::Track => {
                Self::_sort(vec, self.reverse, |s| lib[*s].track())
            }
            OrderField::Disc => {
                if self.simple {
                    Self::_sort(vec, self.reverse, |s| lib[*s].disc());
                } else {
                    Self::_sort(vec, self.reverse, |s| {
                        (lib[*s].disc(), lib[*s].track())
                    })
                }
            }
            OrderField::Album => {
                if self.simple {
                    Self::_sort(vec, self.reverse, |s| lib[*s].album())
                } else {
                    Self::_sort(vec, self.reverse, |s| {
                        (lib[*s].album(), lib[*s].disc(), lib[*s].track())
                    })
                }
            }
            OrderField::Artist => {
                if self.simple {
                    Self::_sort(vec, self.reverse, |s| lib[*s].artist());
                } else {
                    Self::_sort(vec, self.reverse, |s| {
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
                    Self::_sort(vec, self.reverse, |s| lib[*s].year())
                } else {
                    Self::_sort(vec, self.reverse, |s| {
                        (
                            lib[*s].year(),
                            lib[*s].album(),
                            lib[*s].disc(),
                            lib[*s].track(),
                        )
                    })
                }
            }
            OrderField::Length => {
                Self::_sort(vec, self.reverse, |s| lib[*s].length())
            }
            OrderField::Genre => {
                if self.simple {
                    Self::_sort(vec, self.reverse, |s| lib[*s].genre());
                } else {
                    Self::_sort(vec, self.reverse, |s| {
                        (lib[*s].genre(), lib[*s].year())
                    });
                }
            }
        }
    }

    fn _sort<F, O>(vec: &mut [SongId], reverse: bool, f: F)
    where
        O: Ord,
        F: Fn(&SongId) -> O,
    {
        if reverse {
            vec.sort_by_key(|v| Reverse(f(v)));
        } else {
            vec.sort_by_key(f);
        }
    }
}
