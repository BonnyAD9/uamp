mod base;
mod composed_filter;
mod filter;
mod lexer;
mod order;
mod parser;
mod unique;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

use std::{fmt::Display, str::FromStr};

use itertools::Itertools;
use pareg::{ArgError, FromArgStr};
use parser::Parser;
use serde::{Deserialize, Serialize};

use crate::core::{Result, player::Player, query::unique::Unique};

pub use self::{base::*, composed_filter::*, filter::*, order::*};

use super::library::{Library, Song, SongId};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Query {
    bases: Vec<Base>,
    filter: ComposedFilter,
    sort: Option<SongOrder>,
    unique: Option<Unique>,
}

impl Query {
    pub fn new(
        bases: Vec<Base>,
        filter: ComposedFilter,
        sort: Option<SongOrder>,
        unique: Option<Unique>,
    ) -> Self {
        Self {
            bases,
            filter,
            sort,
            unique,
        }
    }

    pub fn all_rng() -> Self {
        Self::new(
            vec![],
            ComposedFilter::Filter(Filter::any()),
            Some(SongOrder::rng()),
            None,
        )
    }

    pub fn get_ids(
        &self,
        lib: &Library,
        simple: bool,
        player: &Player,
    ) -> Result<Vec<SongId>> {
        let mut buf = String::new();

        let mut bases = vec![];
        for i in &self.bases {
            bases.push(i.iter(lib, player)?);
        }

        if bases.is_empty() {
            bases.push(Base::default().iter(lib, player)?);
        }

        let mut res = bases
            .into_iter()
            .flatten()
            .filter(|i| self.filter.matches(&lib[i], &mut buf))
            .collect_vec();

        if let Some(s) = self.sort {
            s.sort(lib, &mut res[..], simple, None);
        }

        if let Some(u) = self.unique {
            u.filter_id(&mut res, lib);
        }

        Ok(res)
    }

    pub fn clone_songs(
        &self,
        lib: &Library,
        simple: bool,
        player: &Player,
    ) -> Result<Vec<Song>> {
        let mut res = self
            .get_ids(lib, simple, player)?
            .into_iter()
            .map(|a| lib[a].clone())
            .collect_vec();
        if let Some(u) = self.unique {
            u.filter_song(&mut res);
        }
        Ok(res)
    }
}

impl FromStr for Query {
    type Err = ArgError;

    fn from_str(s: &str) -> pareg::Result<Self> {
        Parser::parse_query(s)
    }
}

impl FromArgStr for Query {}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.bases.is_empty() {
            for b in &self.bases {
                write!(f, ",{b}")?;
            }
            write!(f, "@")?;
        }
        write!(f, "{}", self.filter)?;
        if let Some(o) = self.sort {
            write!(f, "@{o}")?;
        }
        Ok(())
    }
}
