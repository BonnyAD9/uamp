use std::{fmt::Display, str::FromStr};

use pareg::{ArgError, ArgInto, FromArgStr};
use serde::{Deserialize, Serialize};

use crate::{core::library::Song, ext::simpl};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Filter for searching in songs.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Filter {
    typ: FilterType,
    cmp: CmpType,
    negate: bool,
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
    AnyName(Option<String>),
    /// Song title contains the given string.
    Title(Option<String>),
    /// Song artist contains the given string.
    Artist(Option<String>),
    /// Song album contains the given string.
    Album(Option<String>),
    /// Song album artist matches given string.
    AlbumArtist(Option<String>),
    /// Track number has the given value.
    Track(Option<u32>),
    /// Disc number has the given value.
    Disc(Option<u32>),
    /// Song was released within the given year.
    Year(Option<i32>),
    /// Song genre contains the given string.
    Genre(Option<String>),
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
    pub fn new(mut typ: FilterType, cmp: CmpType, negate: bool) -> Self {
        typ.prepare(cmp);
        Self { cmp, typ, negate }
    }

    /// Check if the given song passes the filter.
    pub fn matches(&self, song: &Song, buf: &mut String) -> bool {
        self.typ.matches(song, self.cmp, buf) ^ self.negate
    }

    pub fn none() -> Self {
        Self::new(FilterType::None, CmpType::Lenient, false)
    }

    pub fn any() -> Self {
        Self::new(FilterType::Any, CmpType::Lenient, false)
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
        macro_rules! eqo {
            ($c:expr, $s:expr) => {
                match ($c, $s) {
                    (None, None) => true,
                    (Some(c), Some(s)) => cmp.matches(c, s, buf),
                    _ => false,
                }
            };
        }

        macro_rules! eqs {
            ($c:expr, $s:expr) => {
                match ($c, $s) {
                    (None, []) => true,
                    (Some(c), s) => s.iter().any(|s| cmp.matches(c, s, buf)),
                    _ => false,
                }
            };
        }

        match self {
            Self::Any => true,
            Self::None => false,
            Self::AnyName(s) => {
                eqo!(s, song.title())
                    || eqs!(s, song.artists())
                    || eqo!(s, song.album())
            }
            Self::Title(s) => eqo!(s, song.title()),
            Self::Artist(s) => eqs!(s, song.artists()),
            Self::Album(s) => eqo!(s, song.album()),
            Self::AlbumArtist(s) => eqo!(s, song.album_artist()),
            Self::Track(t) => *t == song.track(),
            Self::Disc(d) => *d == song.disc(),
            Self::Year(y) => *y == song.year(),
            Self::Genre(s) => eqs!(s, song.genres()),
        }
    }

    /// Prepare the filter. This must be called before any call to `matches`.
    pub fn prepare(&mut self, cmp: CmpType) {
        if cmp.is_strict() {
            return;
        }

        match self {
            Self::AnyName(s) => *s = s.as_deref().map(simpl::new_str),
            Self::Title(s) => *s = s.as_deref().map(simpl::new_str),
            Self::Artist(s) => *s = s.as_deref().map(simpl::new_str),
            Self::Album(s) => *s = s.as_deref().map(simpl::new_str),
            Self::AlbumArtist(s) => *s = s.as_deref().map(simpl::new_str),
            Self::Genre(s) => *s = s.as_deref().map(simpl::new_str),
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
            simpl::to_str(s.as_ref(), buf);
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
        let n = if self.negate { "^" } else { "" };
        let c = match self.cmp {
            CmpType::Strict => '=',
            CmpType::StrictContains => '-',
            CmpType::Lenient => ':',
            CmpType::LenientContains => '~',
        };

        match &self.typ {
            FilterType::Any => f.write_str("any"),
            FilterType::None => f.write_str("none"),
            FilterType::AnyName(None) => write!(f, "s{c}"),
            FilterType::AnyName(Some(a)) => write!(f, "s{n}{c}{a}"),
            FilterType::Title(None) => write!(f, "n{c}"),
            FilterType::Title(Some(t)) => write!(f, "n{n}{c}{t}"),
            FilterType::Artist(None) => write!(f, "p{n}{c}"),
            FilterType::Artist(Some(a)) => write!(f, "p{n}{c}{a}"),
            FilterType::Album(None) => write!(f, "a{n}{c}"),
            FilterType::Album(Some(a)) => write!(f, "a{n}{c}{a}"),
            FilterType::AlbumArtist(None) => write!(f, "aa{n}{c}"),
            FilterType::AlbumArtist(Some(a)) => write!(f, "aa{n}{c}{a}"),
            FilterType::Track(None) => write!(f, "t{n}{c}"),
            FilterType::Track(Some(t)) => write!(f, "t{n}{c}{t}"),
            FilterType::Disc(None) => write!(f, "d{n}{c}"),
            FilterType::Disc(Some(d)) => write!(f, "d{n}{c}{d}"),
            FilterType::Year(None) => write!(f, "y{n}{c}"),
            FilterType::Year(Some(y)) => write!(f, "y{n}{c}{y}"),
            FilterType::Genre(None) => write!(f, "g{n}{c}"),
            FilterType::Genre(Some(g)) => write!(f, "g{n}{c}{g}"),
        }
    }
}

impl FromStr for Filter {
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((p, mut c)) = s
            .char_indices()
            .find(|(_, c)| matches!(c, '=' | '+' | ':' | '~' | '^'))
        else {
            return match s {
                "any" => Ok(Self::new(FilterType::Any, CmpType::default(), false)),
                "none" => Ok(Self::new(FilterType::None, CmpType::default(), false)),
                "s" | "an" | "any-name" | "n" | "tit" | "title" | "name"
                | "p" | "art" | "artist" | "performer" | "auth" | "author"
                | "a" | "alb" | "album" | "t" | "trk" | "track" | "album_artist" | "aa" | "ap"
                | "track-number" | "d" | "disc" | "y" | "year" | "g"
                | "genre" => ArgError::failed_to_parse(
                    "Missing argument for filter.",
                    s,
                )
                .spanned(s.len()..s.len())
                .hint(
                    "Use `=`, `+`, `:` or `~` and add argument to the filter.",
                )
                .err(),
                v => ArgError::failed_to_parse(
                    format!("Unknown filter type `{v}`."),
                    s,
                )
                .err(),
            };
        };

        let negate = c == '^';
        let mut skip = c.len_utf8();
        if negate {
            let Some(chr) = &s[p + skip..].chars().next() else {
                return ArgError::failed_to_parse(
                    "Missing comparison operator after negation.",
                    s,
                )
                .err();
            };
            c = *chr;
            skip += c.len_utf8();
        }

        let typ = &s[..p];
        let val = &s[p + skip..];

        let cmp = match c {
            '=' => CmpType::Strict,
            '-' => CmpType::StrictContains,
            ':' => CmpType::Lenient,
            '~' => CmpType::LenientContains,
            _ => {
                return ArgError::failed_to_parse(
                    "Invalid comparison operator `{c}`.",
                    s,
                )
                .err();
            }
        };

        let em =
            |e: ArgError| e.shift_span(s.len() - val.len(), s.to_string());

        match typ {
            "s" | "an" | "any-name" => Ok(Self::new(
                FilterType::AnyName(val.arg_into().map_err(em)?),
                cmp,
                negate,
            )),
            "n" | "tit" | "title" | "name" => Ok(Self::new(
                FilterType::Title(val.arg_into().map_err(em)?),
                cmp,
                negate,
            )),
            "p" | "art" | "artist" | "performer" | "auth" | "author" => {
                Ok(Self::new(
                    FilterType::Artist(val.arg_into().map_err(em)?),
                    cmp,
                    negate,
                ))
            }
            "a" | "alb" | "album" => Ok(Self::new(
                FilterType::Album(val.arg_into().map_err(em)?),
                cmp,
                negate,
            )),
            "aa" | "ap" | "album-artist" => Ok(Self::new(
                FilterType::AlbumArtist(val.arg_into().map_err(em)?),
                cmp,
                negate,
            )),
            "t" | "trk" | "track" | "track-number" => Ok(Self::new(
                FilterType::Track(val.arg_into().map_err(em)?),
                cmp,
                negate,
            )),
            "d" | "disc" => Ok(Self::new(
                FilterType::Disc(val.arg_into().map_err(em)?),
                cmp,
                negate,
            )),
            "y" | "year" => Ok(Self::new(
                FilterType::Year(val.arg_into().map_err(em)?),
                cmp,
                negate,
            )),
            "g" | "genre" => Ok(Self::new(
                FilterType::Genre(val.arg_into().map_err(em)?),
                cmp,
                negate,
            )),
            v => ArgError::failed_to_parse(
                format!("Unknown filter type `{v}`."),
                s,
            )
            .spanned(0..p)
            .err(),
        }
    }
}

impl FromArgStr for Filter {}
