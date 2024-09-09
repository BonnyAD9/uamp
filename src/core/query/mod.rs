mod composed_filter;
mod filter;
mod lexer;
mod order;
mod parser;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

use std::{fmt::Display, str::FromStr};

use itertools::Itertools;
use pareg::FromArgStr;
use parser::Parser;
use serde::{Deserialize, Serialize};

pub use self::{composed_filter::*, filter::*, order::*};

use super::{
    library::{Library, Song, SongId},
    Error,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Query {
    filter: ComposedFilter,
    sort: Option<SongOrder>,
}

impl Query {
    pub fn new(filter: ComposedFilter, sort: Option<SongOrder>) -> Self {
        Self { filter, sort }
    }

    pub fn get_ids(
        &self,
        lib: &Library,
        simple: bool,
        iter: impl Iterator<Item = SongId>,
    ) -> Vec<SongId> {
        let mut buf = String::new();
        let mut res = iter
            .filter(|i| self.filter.matches(&lib[*i], &mut buf))
            .collect_vec();

        if let Some(s) = self.sort {
            s.sort(lib, &mut res[..], simple, None);
        }

        res
    }

    pub fn clone_songs(
        &self,
        lib: &Library,
        simple: bool,
        iter: impl Iterator<Item = SongId>,
    ) -> Vec<Song> {
        self.get_ids(lib, simple, iter)
            .into_iter()
            .map(|a| lib[a].clone())
            .collect()
    }
}

impl FromStr for Query {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Parser::parse_query(s)
    }
}

impl FromArgStr for Query {}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.filter)?;
        if let Some(o) = self.sort {
            write!(f, "@{o}")?;
        }
        Ok(())
    }
}
