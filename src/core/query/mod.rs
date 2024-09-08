mod filter;
mod lexer;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

use std::{fmt::Display, mem, str::FromStr};

use itertools::Itertools;
use lexer::{Lexer, Token};
use pareg::{ArgError, FromArgStr};
use serde::{Deserialize, Serialize};

pub use self::filter::*;

use super::{library::Song, Error, Result};

// [name~/mix/+auth:imaginedragons].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Query {
    Filter(Filter),
    And(Vec<Query>),
    Or(Vec<Query>),
}

impl Query {
    pub fn matches(&self, song: &Song, buf: &mut String) -> bool {
        match self {
            Query::Filter(f) => f.matches(song, buf),
            Query::And(q) => {
                for q in q {
                    if !q.matches(song, buf) {
                        return false;
                    }
                }
                true
            }
            Query::Or(q) => {
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

impl FromStr for Query {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Parser::parse(s)
    }
}

impl FromArgStr for Query {}

impl Default for Query {
    fn default() -> Self {
        Query::Or(Default::default())
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Query::Filter(ft) => {
                f.write_str(&ft.to_string().replace('/', "//"))
            }
            Query::And(a) => {
                write!(f, "[{}]", a.iter().map(|a| a.to_string()).join("."))
            }
            Query::Or(o) => {
                write!(f, "[{}]", o.iter().map(|a| a.to_string()).join("+"))
            }
        }
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

struct Parser<'a> {
    lex: Lexer<'a>,
    cur: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn parse(s: &'a str) -> Result<Query> {
        Self {
            lex: Lexer::new(s),
            cur: None,
        }
        .parse_inner()
    }

    fn parse_inner(&mut self) -> Result<Query> {
        let r = self.parse_or()?;
        if let Some(t) = self.cur()? {
            Err(ArgError::FailedToParse {
                typ: "query",
                value: t.to_string().into(),
                msg: Some("Unused input".into()),
            })?;
        }
        Ok(r)
    }

    fn parse_or(&mut self) -> Result<Query> {
        let mut exprs = vec![];

        while let Some(t) = self.cur()? {
            match t {
                Token::Filter(_) => exprs.push(self.parse_and()?),
                Token::Open => {
                    exprs.push(self.parse_bracket()?);
                }
                t => Err(ArgError::FailedToParse {
                    typ: "query",
                    value: t.to_string().into(),
                    msg: Some("Expected filter or '['".into()),
                })?,
            }

            let Some(t) = self.cur()? else {
                break;
            };

            match t {
                Token::Or => _ = self.next()?,
                Token::Close => break,
                t => Err(ArgError::FailedToParse {
                    typ: "query",
                    value: t.to_string().into(),
                    msg: Some("Expected filter or '['".into()),
                })?,
            }
        }

        match &mut exprs[..] {
            [] => Ok(Query::default()),
            [e] => Ok(mem::take(e)),
            _ => Ok(Query::Or(exprs)),
        }
    }

    fn parse_bracket(&mut self) -> Result<Query> {
        self.next()?;
        let r = self.parse_and()?;
        if !matches!(self.cur()?, Some(Token::Close)) {
            Err(ArgError::FailedToParse {
                typ: "query",
                value: self
                    .cur
                    .as_ref()
                    .map(|c| c.to_string())
                    .unwrap_or("None".to_owned())
                    .into(),
                msg: Some("Expected ']'".into()),
            })?
        }
        self.next()?;
        Ok(r)
    }

    fn parse_and(&mut self) -> Result<Query> {
        let mut exprs = vec![];

        while let Some(t) = self.cur()? {
            match t {
                Token::Filter(_) => exprs.push(self.parse_filter()?),
                Token::Open => {
                    exprs.push(self.parse_bracket()?);
                }
                t => Err(ArgError::FailedToParse {
                    typ: "query",
                    value: t.to_string().into(),
                    msg: Some("Expected filter or '['".into()),
                })?,
            }

            let Some(t) = self.cur()? else {
                break;
            };

            match t {
                Token::And => _ = self.next()?,
                Token::Close | Token::Or => break,
                t => Err(ArgError::FailedToParse {
                    typ: "query",
                    value: t.to_string().into(),
                    msg: Some("Expected filter or '['".into()),
                })?,
            }
        }

        match &mut exprs[..] {
            [e] => Ok(mem::take(e)),
            _ => Ok(Query::And(exprs)),
        }
    }

    fn parse_filter(&mut self) -> Result<Query> {
        self.cur()?;
        let f = match self.cur.take() {
            Some(Token::Filter(f)) => f,
            t => Err(ArgError::FailedToParse {
                typ: "query",
                value: t
                    .map(|c| c.to_string())
                    .unwrap_or("None".to_owned())
                    .into(),
                msg: Some("Expected filter".into()),
            })?,
        };

        Ok(Query::Filter(f))
    }

    fn next(&mut self) -> Result<Option<&Token>> {
        self.cur = self.lex.next()?;
        Ok(self.cur.as_ref())
    }

    fn cur(&mut self) -> Result<Option<&Token>> {
        if let Some(ref c) = self.cur {
            Ok(Some(c))
        } else {
            self.next()
        }
    }
}
