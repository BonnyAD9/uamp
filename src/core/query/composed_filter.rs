use std::{fmt::Display, str::FromStr};

use itertools::Itertools;
use pareg::FromArgStr;
use serde::{Deserialize, Serialize};

use crate::core::{library::Song, Error, Result};

use super::{parser::Parser, Filter};

// [name~/mix/+auth:imaginedragons].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComposedFilter {
    Filter(Filter),
    And(Vec<ComposedFilter>),
    Or(Vec<ComposedFilter>),
}

impl ComposedFilter {
    pub fn matches(&self, song: &Song, buf: &mut String) -> bool {
        match self {
            ComposedFilter::Filter(f) => f.matches(song, buf),
            ComposedFilter::And(q) => {
                for q in q {
                    if !q.matches(song, buf) {
                        return false;
                    }
                }
                true
            }
            ComposedFilter::Or(q) => {
                for q in q {
                    if q.matches(song, buf) {
                        return true;
                    }
                }
                false
            }
        }
    }
}

impl FromStr for ComposedFilter {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Parser::parse_composed_filter(s)
    }
}

impl FromArgStr for ComposedFilter {}

impl Default for ComposedFilter {
    fn default() -> Self {
        ComposedFilter::And(Default::default())
    }
}

impl Display for ComposedFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComposedFilter::Filter(ft) => {
                f.write_str(&ft.to_string().replace('/', "//"))
            }
            ComposedFilter::And(a) => {
                write!(f, "[{}]", a.iter().map(|a| a.to_string()).join("."))
            }
            ComposedFilter::Or(o) => {
                write!(f, "[{}]", o.iter().map(|a| a.to_string()).join("+"))
            }
        }
    }
}
