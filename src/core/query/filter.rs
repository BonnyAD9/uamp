use std::{fmt::Display, str::FromStr};

use pareg::{ArgError, FromArgStr};
use serde::{Deserialize, Serialize};
use unidecode::unidecode_char;

use crate::core::{library::Song, Error};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Filter for searching in songs.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Filter {
    cmp: CmpType,
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

#[derive(
    Clone, Debug, Serialize, Deserialize, Default, Copy, Eq, PartialEq,
)]
pub enum CmpType {
    // =
    Strict,
    // /
    StrictContains,
    // :
    Lenient,
    // ~
    #[default]
    LenientContains,
}

impl Filter {
    /// Creates new filter.
    pub fn new(mut typ: FilterType, cmp: CmpType) -> Self {
        typ.prepare(cmp);
        Self { cmp, typ }
    }

    /// Check if the given song passes the filter.
    pub fn matches(&self, song: &Song, buf: &mut String) -> bool {
        self.typ.matches(song, self.cmp, buf)
    }

    pub fn none() -> Self {
        Self::new(FilterType::None, CmpType::Lenient)
    }
}

impl FilterType {
    /// Checks if the given song passes the filter.
    ///
    /// - `buf` is temporary strorage used for comparisons.
    pub fn matches(
        &self,
        song: &Song,
        cmp: CmpType,
        buf: &mut String,
    ) -> bool {
        let mut eq = |c, s| cmp.matches(c, s, buf);

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
    pub fn prepare(&mut self, cmp: CmpType) {
        if cmp.is_strict() {
            return;
        }

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

impl CmpType {
    pub fn is_lenient(&self) -> bool {
        matches!(self, CmpType::Lenient | CmpType::LenientContains)
    }

    pub fn is_contains(&self) -> bool {
        matches!(self, CmpType::StrictContains | CmpType::LenientContains)
    }

    pub fn is_strict(&self) -> bool {
        !self.is_lenient()
    }

    pub fn matches(
        &self,
        pat: impl AsRef<str>,
        s: impl AsRef<str>,
        buf: &mut String,
    ) -> bool {
        let s = if self.is_lenient() {
            buf.clear();
            cache_str(s.as_ref(), buf);
            buf.as_str()
        } else {
            s.as_ref()
        };

        if self.is_contains() {
            s.contains(pat.as_ref())
        } else {
            *pat.as_ref() == *s
        }
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self.cmp {
            CmpType::Strict => '=',
            CmpType::StrictContains => '+',
            CmpType::Lenient => ':',
            CmpType::LenientContains => '~',
        };

        match &self.typ {
            FilterType::Any => f.write_str("any"),
            FilterType::None => f.write_str("none"),
            FilterType::AnyName(n) => write!(f, "s{c}{n}"),
            FilterType::Title(t) => write!(f, "n{c}{t}"),
            FilterType::Artist(a) => write!(f, "p{c}{a}"),
            FilterType::Album(a) => write!(f, "a{c}{a}"),
            FilterType::Track(t) => write!(f, "t{c}{t}"),
            FilterType::Disc(d) => write!(f, "d{c}{d}"),
            FilterType::Year(y) => write!(f, "y{c}{y}"),
            FilterType::Genre(g) => write!(f, "g{c}{g}"),
        }
    }
}
impl FromStr for Filter {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((p, c)) = s
            .char_indices()
            .find(|(_, c)| matches!(c, '=' | '+' | ':' | '~'))
        else {
            return match s {
                "any" => Ok(Self::new(FilterType::Any, CmpType::default())),
                "none" => Ok(Self::new(FilterType::None, CmpType::default())),
                v => {
                    Err(ArgError::UnknownArgument(v.to_owned().into()).into())
                }
            };
        };
        let typ = &s[..p];
        let val = &s[p + c.len_utf8()..];
        let cmp = match c {
            '=' => CmpType::Strict,
            '+' => CmpType::StrictContains,
            ':' => CmpType::Lenient,
            '~' => CmpType::LenientContains,
            _ => unreachable!(),
        };

        match typ {
            "s" | "an" | "any-name" => {
                Ok(Self::new(FilterType::AnyName(val.to_owned()), cmp))
            }
            "n" | "tit" | "title" | "name" => {
                Ok(Self::new(FilterType::Title(val.to_owned()), cmp))
            }
            "p" | "art" | "artist" | "performer" | "auth" | "author" => {
                Ok(Self::new(FilterType::Artist(val.to_owned()), cmp))
            }
            "a" | "alb" | "album" => {
                Ok(Self::new(FilterType::Album(val.to_owned()), cmp))
            }
            "t" | "trk" | "track" | "track-number" => {
                Ok(Self::new(FilterType::Track(val.parse()?), cmp))
            }
            "d" | "disc" => Ok(Self::new(FilterType::Disc(val.parse()?), cmp)),
            "y" | "year" => Ok(Self::new(FilterType::Year(val.parse()?), cmp)),
            "g" | "genre" => {
                Ok(Self::new(FilterType::Genre(val.to_owned()), cmp))
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
