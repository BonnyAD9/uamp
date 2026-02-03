use std::{
    borrow::{Borrow, Cow},
    collections::HashSet,
    hash::Hash,
    time::Duration,
};

use pareg::FromArg;
use serde::{Deserialize, Serialize};

use crate::core::library::{Library, Song, SongId};

#[derive(Debug, Copy, Clone, FromArg, Serialize, Deserialize)]
pub enum Unique {
    #[arg("id" | "path" | "unique" | "u")]
    Songs,
    #[arg("n" | "tit" | "name")]
    Title,
    #[arg("p" | "art" | "performer" | "auth" | "author")]
    Artist,
    #[arg("a" | "alb")]
    Album,
    #[arg("t" | "trk" | "track-number")]
    Track,
    #[arg("d")]
    Disc,
    #[arg("y")]
    Year,
    #[arg("len")]
    Length,
    #[arg("g")]
    Genre,
}

impl Unique {
    pub fn filter_song(&self, data: &mut Vec<Song>) {
        match self {
            Self::Songs => unique_filter_song(data, |a| a.path().into()),
            Self::Title => unique_filter_song(data, |a| a.title_str().into()),
            Self::Artist => unique_filter_song(data, |a| a.artists().into()),
            Self::Album => unique_filter_song(data, |a| a.album_str().into()),
            Self::Track => unique_filter_song(data, |a| {
                Cow::<u32>::Owned(a.track().unwrap_or_default())
            }),
            Self::Disc => unique_filter_song(data, |a| {
                Cow::<u32>::Owned(a.disc().unwrap_or_default())
            }),
            Self::Year => unique_filter_song(data, |a| {
                Cow::<i32>::Owned(a.year().unwrap_or_default())
            }),
            Self::Length => unique_filter_song(data, |a| {
                Cow::<Duration>::Owned(a.length().unwrap_or_default())
            }),
            Self::Genre => unique_filter_song(data, |a| a.genres().into()),
        }
    }

    pub fn filter_id(&self, data: &mut Vec<SongId>, lib: &Library) {
        match self {
            Self::Songs => unique_filter_id(data, Cow::<SongId>::Owned),
            Self::Title => {
                unique_filter_id(data, |a| lib[a].title_str().into())
            }
            Self::Artist => {
                unique_filter_id(data, |a| lib[a].artists().into())
            }
            Self::Album => {
                unique_filter_id(data, |a| lib[a].album_str().into())
            }
            Self::Track => unique_filter_id(data, |a| {
                Cow::<u32>::Owned(lib[a].track().unwrap_or_default())
            }),
            Self::Disc => unique_filter_id(data, |a| {
                Cow::<u32>::Owned(lib[a].disc().unwrap_or_default())
            }),
            Self::Year => unique_filter_id(data, |a| {
                Cow::<i32>::Owned(lib[a].year().unwrap_or_default())
            }),
            Self::Length => unique_filter_id(data, |a| {
                Cow::<Duration>::Owned(lib[a].length().unwrap_or_default())
            }),
            Self::Genre => unique_filter_id(data, |a| lib[a].genres().into()),
        }
    }
}

fn unique_filter_song<
    S,
    Q: Hash + Eq + ToOwned<Owned = T> + ?Sized,
    T: Hash + Eq + Borrow<Q>,
>(
    data: &mut Vec<S>,
    p: impl Fn(&S) -> Cow<Q>,
) {
    let mut used = HashSet::new();
    data.retain(|s| {
        let attr = p(s);
        if used.contains(attr.as_ref()) {
            false
        } else {
            used.insert(attr.into_owned());
            true
        }
    });
}

fn unique_filter_id<
    'a,
    Q: Hash + Eq + ToOwned<Owned = T> + ?Sized + 'a,
    T: Hash + Eq + Borrow<Q>,
>(
    data: &mut Vec<SongId>,
    p: impl Fn(SongId) -> Cow<'a, Q> + 'a,
) {
    let mut used = HashSet::new();
    data.retain(|s| {
        let attr = p(*s);
        if used.contains(attr.as_ref()) {
            false
        } else {
            used.insert(attr.into_owned());
            true
        }
    });
}
