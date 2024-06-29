use std::{fmt::Display, str::FromStr};

use pareg::{starts_any, val_arg, ArgError, FromArgStr};
use serde::{Deserialize, Serialize};
use unidecode::unidecode_char;

use crate::core::{library::Song, Error};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Filter for searching in songs.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Filter {
    typ: FilterType,
}

/// Filter type.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum FilterType {
    /// Everything passes this filter.
    #[default]
    Any,
    /// Nothing passes this filter.
    None,
    /// Song title, artist or album name contains the given string.
    AnyName(String),
    /// Song title contains the given string.
    Title(String),
    /// Song artist contains the given string.
    Artist(String),
    /// Song album contains the given string.
    Album(String),
    /// Track number has the given value.
    Track(u32),
    /// Disc number has the given value.
    Disc(u32),
    /// Song was released within the given year.
    Year(i32),
    /// Song genre contains the given string.
    Genre(String),
}

impl Filter {
    /// Creates new filter.
    pub fn new(mut typ: FilterType) -> Self {
        typ.prepare();
        Self { typ }
    }

    /// Check if the given song passes the filter.
    pub fn matches(&self, song: &Song, buf: &mut String) -> bool {
        self.typ.matches(song, buf)
    }
}

impl FilterType {
    /// Checks if the given song passes the filter.
    ///
    /// - `buf` is temporary strorage used for comparisons.
    pub fn matches(&self, song: &Song, buf: &mut String) -> bool {
        let mut eq = |c, s| {
            buf.clear();
            cache_str(s, buf);
            buf == c
        };

        match self {
            Self::Any => true,
            Self::None => false,
            Self::AnyName(s) => {
                eq(s, song.title())
                    || eq(s, song.artist())
                    || eq(s, song.album())
            }
            Self::Title(s) => eq(s, song.title()),
            Self::Artist(s) => eq(s, song.artist()),
            Self::Album(s) => eq(s, song.album()),
            Self::Track(t) => *t == song.track(),
            Self::Disc(d) => *d == song.disc(),
            Self::Year(y) => *y == song.year(),
            Self::Genre(s) => eq(s, song.genre()),
        }
    }

    /// Prepare the filter. This must be called before any call to `matches`.
    pub fn prepare(&mut self) {
        match self {
            Self::AnyName(s) => *s = cache_new_str(s),
            Self::Title(s) => *s = cache_new_str(s),
            Self::Artist(s) => *s = cache_new_str(s),
            Self::Album(s) => *s = cache_new_str(s),
            Self::Genre(s) => *s = cache_new_str(s),
            _ => {}
        }
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.typ)
    }
}

impl Display for FilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any => f.write_str("any"),
            Self::None => f.write_str("none"),
            Self::AnyName(n) => write!(f, "an:{n}"),
            Self::Title(t) => write!(f, "tit:{t}"),
            Self::Artist(a) => write!(f, "art:{a}"),
            Self::Album(a) => write!(f, "alb:{a}"),
            Self::Track(t) => write!(f, "trk:{t}"),
            Self::Disc(d) => write!(f, "disc:{d}"),
            Self::Year(y) => write!(f, "y:{y}"),
            Self::Genre(g) => write!(f, "g:{g}"),
        }
    }
}

impl FromStr for Filter {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.parse()?))
    }
}

impl FromStr for FilterType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "any" => Ok(Self::Any),
            "none" => Ok(Self::None),
            v if starts_any!(v, "an:", "any-name:") => {
                Ok(Self::AnyName(val_arg(v, ':')?))
            }
            v if starts_any!(v, "tit:", "title:", "name:") => {
                Ok(Self::Title(val_arg(v, ':')?))
            }
            v if starts_any!(
                v,
                "art:",
                "artist:",
                "performer:",
                "auth:",
                "author:",
            ) =>
            {
                Ok(Self::Artist(val_arg(v, ':')?))
            }
            v if starts_any!(v, "alb:", "album:") => {
                Ok(Self::Album(val_arg(v, ':')?))
            }
            v if starts_any!(v, "trk:", "track:", "track-number:") => {
                Ok(Self::Track(val_arg(v, ':')?))
            }
            v if starts_any!(v, "disc:") => Ok(Self::Disc(val_arg(v, ':')?)),
            v if starts_any!(v, "y:", "year:") => {
                Ok(Self::Year(val_arg(v, ':')?))
            }
            v if starts_any!(v, "g:", "genre:") => {
                Ok(Self::Genre(val_arg(v, ':')?))
            }
            v => Err(ArgError::UnknownArgument(v.to_owned().into()).into()),
        }
    }
}

impl FromArgStr for Filter {}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

fn cache_new_str(s: &str) -> String {
    let mut res = String::new();
    cache_str(s, &mut res);
    res
}

fn cache_str(s: &str, out: &mut String) {
    for s in s.chars().map(|c| unidecode_char(c).to_ascii_lowercase()) {
        out.extend(s.chars().filter(|a| !a.is_ascii_whitespace()))
    }
}
