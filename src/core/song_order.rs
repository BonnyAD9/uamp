use std::{
    cmp::Reverse,
    fmt::{Display, Write},
    str::FromStr,
};

use pareg::{ArgError, FromArgStr};
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

use super::{
    library::{Library, SongId},
    Error,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Describes how to order songs.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SongOrder {
    /// Enable/Disable simple ordering. When using simple ordering, the songs
    /// are ordered only by the main field.
    ///
    /// When [`None`] use the default value.
    pub simple: Option<bool>,
    /// The main field to order by.
    pub field: OrderField,
    /// When `true`, reverse after the sorting.
    pub reverse: bool,
}

/// Describes the main ordering field.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderField {
    /// Just reverse.
    Reverse,
    /// Randomly shuffle.
    Randomize,
    /// Order by file path.
    Path,
    /// Order by song title.
    Title,
    /// Order by song artist.
    Artist,
    /// Order by song album.
    Album,
    /// Order by the track number.
    Track,
    /// Order by the disc number.
    Disc,
    /// Order by the release date.
    Year,
    /// Order by total track length.
    Length,
    /// Order by the genre.
    Genre,
}

impl SongOrder {
    /// Sorts the songs and updates the position of cur if set.
    ///
    /// - `songs`: Songs to order.
    /// - `simple`: Defaule value for *enable simple sorting* when it is unset.
    /// - `cur`: Index to the array of cur. When [`Some`] it is updated after
    ///   the sorting so that it points to the same song.
    pub fn sort(
        &self,
        lib: &Library,
        songs: &mut [SongId],
        simple: bool,
        cur: Option<&mut usize>,
    ) {
        let cur = if let Some(cur) = cur {
            let cur_song = songs[*cur];
            Some((cur, cur_song))
        } else {
            None
        };

        match self.field {
            OrderField::Reverse => self.reverse(songs),
            OrderField::Randomize => self.randomize(songs),
            OrderField::Path => self.path(lib, songs),
            OrderField::Title => self.title(lib, songs),
            OrderField::Artist => self.artist(lib, simple, songs),
            OrderField::Album => self.album(lib, simple, songs),
            OrderField::Track => self.track(lib, songs),
            OrderField::Disc => self.disc(lib, simple, songs),
            OrderField::Year => self.year(lib, simple, songs),
            OrderField::Length => self.length(lib, songs),
            OrderField::Genre => self.genre(lib, songs),
        }

        if let Some((idx, song)) = cur {
            if songs[*idx] != song {
                *idx = songs.iter().position(|s| *s == song).unwrap();
            }
        }
    }
}

impl FromStr for SongOrder {
    type Err = Error;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let mut reverse = false;
        if let Some(rest) = s.strip_prefix('-') {
            s = rest;
            reverse = true;
        }

        let field = match s {
            "rev" | "reverse" => OrderField::Reverse,
            "rng" | "rand" | "random" | "randomize" => OrderField::Randomize,
            "path" => OrderField::Path,
            "title" | "name" => OrderField::Title,
            "artist" | "performer" | "author" => OrderField::Artist,
            "album" => OrderField::Album,
            "track" => OrderField::Track,
            "disc" => OrderField::Disc,
            "year" | "date" => OrderField::Year,
            "len" | "length" => OrderField::Length,
            "genre" => OrderField::Genre,
            _ => {
                return Err(Error::ArgParse(ArgError::FailedToParse {
                    typ: "SongOrder",
                    value: s.to_owned().into(),
                    msg: Some("Invalid enum value.".into()),
                }))
            }
        };

        Ok(Self {
            simple: None,
            reverse,
            field,
        })
    }
}

impl Display for SongOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.reverse {
            f.write_char('-')?;
        }

        match self.field {
            OrderField::Reverse => write!(f, "rev"),
            OrderField::Randomize => write!(f, "rng"),
            OrderField::Path => write!(f, "path"),
            OrderField::Title => write!(f, "title"),
            OrderField::Artist => write!(f, "artist"),
            OrderField::Album => write!(f, "album"),
            OrderField::Track => write!(f, "track"),
            OrderField::Disc => write!(f, "disc"),
            OrderField::Year => write!(f, "date"),
            OrderField::Length => write!(f, "len"),
            OrderField::Genre => write!(f, "genre"),
        }
    }
}

impl FromArgStr for SongOrder {}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl SongOrder {
    fn sort_key<F, O>(&self, songs: &mut [SongId], f: F)
    where
        O: Ord,
        F: Fn(SongId) -> O,
    {
        if self.reverse {
            songs.sort_by_key(|i| Reverse(f(*i)));
        } else {
            songs.sort_by_key(|i| f(*i));
        }
    }

    fn reverse(&self, songs: &mut [SongId]) {
        songs.reverse();
    }

    fn randomize(&self, songs: &mut [SongId]) {
        songs.shuffle(&mut thread_rng());
    }

    fn path(&self, lib: &Library, songs: &mut [SongId]) {
        self.sort_key(songs, |s| lib[s].path());
    }

    fn title(&self, lib: &Library, songs: &mut [SongId]) {
        self.sort_key(songs, |s| lib[s].title());
    }

    fn artist(&self, lib: &Library, simple: bool, songs: &mut [SongId]) {
        if self.simple.unwrap_or(simple) {
            self.sort_key(songs, |s| lib[s].artist());
        } else {
            self.sort_key(songs, |s| {
                (
                    lib[s].artist(),
                    lib[s].year(),
                    lib[s].album(),
                    lib[s].disc(),
                    lib[s].track(),
                )
            })
        }
    }

    fn album(&self, lib: &Library, simple: bool, songs: &mut [SongId]) {
        if self.simple.unwrap_or(simple) {
            self.sort_key(songs, |s| lib[s].album());
        } else {
            self.sort_key(songs, |s| {
                (lib[s].album(), lib[s].disc(), lib[s].track())
            });
        }
    }

    fn track(&self, lib: &Library, songs: &mut [SongId]) {
        self.sort_key(songs, |s| lib[s].track());
    }

    fn disc(&self, lib: &Library, simple: bool, songs: &mut [SongId]) {
        if self.simple.unwrap_or(simple) {
            self.sort_key(songs, |s| lib[s].disc());
        } else {
            self.sort_key(songs, |s| (lib[s].disc(), lib[s].track()));
        }
    }

    fn year(&self, lib: &Library, simple: bool, songs: &mut [SongId]) {
        if self.simple.unwrap_or(simple) {
            self.sort_key(songs, |s| lib[s].year());
        } else {
            self.sort_key(songs, |s| {
                (lib[s].year(), lib[s].album(), lib[s].disc(), lib[s].track())
            });
        }
    }

    fn length(&self, lib: &Library, songs: &mut [SongId]) {
        self.sort_key(songs, |s| lib[s].length());
    }

    fn genre(&self, lib: &Library, songs: &mut [SongId]) {
        self.sort_key(songs, |s| lib[s].genre());
    }
}
