use std::{fmt::Display, str::FromStr};

use pareg::{ArgError, FromArgStr};
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

use crate::library::{Library, SongId};

use super::Error;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SongOrder {
    Reverse,
    Randomize,
    Path,
    Title,
    Artist,
    Album,
    Track,
    Disc,
    Year,
    Length,
    Genre,
}

impl SongOrder {
    pub fn sort(
        &self,
        lib: &Library,
        songs: &mut [SongId],
        simple: bool,
        cur_first: bool,
        cur: Option<&mut usize>,
    ) {
        let cur = if let Some(cur) = cur {
            let cur_song = songs[*cur];
            if cur_first {
                songs[*cur] = songs[0];
                songs[0] = cur_song;
                *cur = 0;
            }
            Some((cur, cur_song))
        } else {
            None
        };

        {
            let songs = if cur.is_some() && cur_first {
                &mut songs[1..]
            } else {
                &mut songs[..]
            };

            if simple {
                self.sort_simple(lib, songs);
            } else {
                self.sort_complex(lib, songs);
            }
        }

        if let Some((idx, song)) = cur {
            if songs[*idx] != song {
                *idx = songs.iter().position(|s| *s == song).unwrap();
            }
        }
    }

    pub fn sort_simple(&self, lib: &Library, songs: &mut [SongId]) {
        match self {
            Self::Reverse => songs.reverse(),
            Self::Randomize => songs.shuffle(&mut thread_rng()),
            Self::Path => songs.sort_by_key(|s| lib[s].path()),
            Self::Title => songs.sort_by_key(|s| lib[s].title()),
            Self::Artist => songs.sort_by_key(|s| lib[s].artist()),
            Self::Album => songs.sort_by_key(|s| lib[s].artist()),
            Self::Track => songs.sort_by_key(|s| lib[s].track()),
            Self::Disc => songs.sort_by_key(|s| lib[s].disc()),
            Self::Year => songs.sort_by_key(|s| lib[s].year()),
            Self::Length => songs.sort_by_key(|s| lib[s].length()),
            Self::Genre => songs.sort_by_key(|s| lib[s].genre()),
        }
    }

    pub fn sort_complex(&self, lib: &Library, songs: &mut [SongId]) {
        match self {
            Self::Reverse => songs.reverse(),
            Self::Randomize => songs.shuffle(&mut thread_rng()),
            Self::Path => songs.sort_by_key(|s| lib[s].path()),
            Self::Title => songs.sort_by_key(|s| lib[s].title()),
            Self::Artist => songs.sort_by_key(|s| {
                (
                    lib[s].artist(),
                    lib[s].year(),
                    lib[s].album(),
                    lib[s].disc(),
                    lib[s].track(),
                )
            }),
            Self::Album => songs.sort_by_key(|s| {
                (lib[s].album(), lib[s].disc(), lib[s].track())
            }),
            Self::Track => songs.sort_by_key(|s| lib[s].track()),
            Self::Disc => songs.sort_by_key(|s| {
                (lib[s].disc(), lib[s].album(), lib[s].track())
            }),
            Self::Year => songs.sort_by_key(|s| {
                (lib[s].year(), lib[s].album(), lib[s].disc(), lib[s].track())
            }),
            Self::Length => songs.sort_by_key(|s| lib[s].length()),
            Self::Genre => songs.sort_by_key(|s| lib[s].genre()),
        }
    }
}

impl FromStr for SongOrder {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rev" | "reverse" => Ok(Self::Reverse),
            "rng" | "rand" | "random" | "randomize" => Ok(Self::Randomize),
            "path" => Ok(Self::Path),
            "title" | "name" => Ok(Self::Title),
            "artist" | "performer" | "author" => Ok(Self::Artist),
            "album" => Ok(Self::Album),
            "track" => Ok(Self::Track),
            "disc" => Ok(Self::Disc),
            "year" | "date" => Ok(Self::Year),
            "len" | "length" => Ok(Self::Length),
            "genre" => Ok(Self::Genre),
            _ => Err(Error::ArgParse(ArgError::FailedToParse {
                typ: "SongOrder",
                value: s.to_owned().into(),
                msg: Some("Invalid enum value.".into()),
            })),
        }
    }
}

impl Display for SongOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reverse => write!(f, "rev"),
            Self::Randomize => write!(f, "rng"),
            Self::Path => write!(f, "path"),
            Self::Title => write!(f, "title"),
            Self::Artist => write!(f, "artist"),
            Self::Album => write!(f, "album"),
            Self::Track => write!(f, "track"),
            Self::Disc => write!(f, "disc"),
            Self::Year => write!(f, "date"),
            Self::Length => write!(f, "len"),
            Self::Genre => write!(f, "genre"),
        }
    }
}

impl FromArgStr for SongOrder {}
