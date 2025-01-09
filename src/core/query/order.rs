use std::{
    cmp::Reverse,
    fmt::{Display, Write},
    str::FromStr,
};

use itertools::PeekingNext;
use pareg::{ArgError, FromArg, FromArgStr};
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

use crate::core::library::{Library, SongId};

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
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, FromArg,
)]
pub enum OrderField {
    /// Don't change the order
    Same,
    /// Just reverse.
    #[arg("rev")]
    Reverse,
    /// Randomly shuffle.
    #[arg("rng" | "rand" | "random")]
    Randomize,
    /// Order by file path.
    Path,
    /// Order by song title.
    #[arg("n" | "tit" | "name")]
    Title,
    /// Order by song artist.
    #[arg("p" | "art" | "performer" | "auth" | "author")]
    Artist,
    /// Order by song album.
    #[arg("a" | "alb")]
    Album,
    /// Order by the track number.
    #[arg("t" | "trk" | "track-number")]
    Track,
    /// Order by the disc number.
    #[arg("d")]
    Disc,
    /// Order by the release date.
    #[arg("y" | "data")]
    Year,
    /// Order by total track length.
    #[arg("len")]
    Length,
    /// Order by the genre.
    #[arg("g")]
    Genre,
}

impl SongOrder {
    pub fn rng() -> Self {
        Self {
            simple: None,
            field: OrderField::Randomize,
            reverse: false,
        }
    }

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
            OrderField::Same => self.same(songs),
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
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut reverse = None;
        let mut simple = None;

        let mut chrs = s.char_indices();
        while let Some((i, c)) =
            chrs.peeking_next(|(_, c)| "<>/\\~+-".contains(*c))
        {
            match c {
                '<' | '>' | '/' | '\\' | '~' => {
                    if reverse.is_some() {
                        return ArgError::parse_msg("Reverse is already set.", s.to_string())
                                .hint("Only one of `<`, `>`, `/`, `\\` or `~` may be specified.")
                                .spanned(i..i + 1)
                                .err();
                    }
                    reverse = Some(matches!(c, '>' | '\\' | '~'));
                }
                '+' | '-' => {
                    if simple.is_some() {
                        return ArgError::parse_msg(
                            "Sort type is already set.",
                            s.to_string(),
                        )
                        .hint("Only one of `+` and `-` are mutualy exclusive.")
                        .spanned(i..i + 1)
                        .err();
                    }
                    simple = Some(c == '-');
                }
                _ => unreachable!(),
            }
        }

        let field = chrs.as_str();
        let field = OrderField::from_arg(field)
            .map_err(|e| e.shift_span(s.len() - field.len(), s.to_string()))?;

        Ok(Self {
            simple,
            reverse: reverse.unwrap_or_default(),
            field,
        })
    }
}

impl Display for SongOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.reverse {
            f.write_char('>')?;
        }

        match self.field {
            OrderField::Same => write!(f, "same"),
            OrderField::Reverse => write!(f, "rev"),
            OrderField::Randomize => write!(f, "rng"),
            OrderField::Path => write!(f, "path"),
            OrderField::Title => write!(f, "n"),
            OrderField::Artist => write!(f, "p"),
            OrderField::Album => write!(f, "a"),
            OrderField::Track => write!(f, "t"),
            OrderField::Disc => write!(f, "d"),
            OrderField::Year => write!(f, "date"),
            OrderField::Length => write!(f, "len"),
            OrderField::Genre => write!(f, "g"),
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

    fn same(&self, songs: &mut [SongId]) {
        if self.reverse {
            songs.reverse();
        }
    }

    fn reverse(&self, songs: &mut [SongId]) {
        if !self.reverse {
            songs.reverse();
        }
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
