use std::mem;

use pareg::ArgError;

use crate::core::Result;

use super::{
    lexer::{Lexer, LexerState, Token},
    ComposedFilter, Query, SongOrder,
};

pub struct Parser<'a> {
    lex: Lexer<'a>,
    cur: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn parse_composed_filter(s: &'a str) -> Result<ComposedFilter> {
        Self {
            lex: Lexer::new(s),
            cur: None,
        }
        .parse_composed_filter_inner()
    }

    pub fn parse_query(s: &'a str) -> Result<Query> {
        Self {
            lex: Lexer::new(s),
            cur: None,
        }
        .parse_query_inner()
    }

    fn parse_composed_filter_inner(&mut self) -> Result<ComposedFilter> {
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

    fn parse_query_inner(&mut self) -> Result<Query> {
        let r = self.parse_q()?;
        if let Some(t) = self.cur()? {
            Err(ArgError::FailedToParse {
                typ: "query",
                value: t.to_string().into(),
                msg: Some("Unused input".into()),
            })?;
        }
        Ok(r)
    }

    fn parse_q(&mut self) -> Result<Query> {
        let filter = self.parse_comp_filter()?;
        if !matches!(self.cur()?, Some(Token::At)) {
            return Ok(Query::new(filter, None));
        }

        self.lex.state = LexerState::Order;
        self.next()?;

        if matches!(self.cur()?, Some(Token::At)) {
            return Ok(Query::new(filter, None));
        }

        let order = self.parse_order()?;

        match self.cur()? {
            Some(Token::At) | None => {}
            Some(t) => {
                Err(ArgError::FailedToParse {
                    typ: "query",
                    value: t.to_string().into(),
                    msg: Some("Unused input".into()),
                })?;
            }
        }

        self.lex.state = LexerState::Filter;

        Ok(Query::new(filter, Some(order)))
    }

    fn parse_order(&mut self) -> Result<SongOrder> {
        self.cur()?;
        match self.cur.take() {
            Some(Token::Order(o)) => Ok(o),
            t => Err(ArgError::FailedToParse {
                typ: "query",
                value: format!("{:?}", t).into(),
                msg: Some("Unused input".into()),
            }
            .into()),
        }
    }

    fn parse_comp_filter(&mut self) -> Result<ComposedFilter> {
        if matches!(self.cur()?, Some(Token::At)) {
            Ok(Default::default())
        } else {
            self.parse_or()
        }
    }

    fn parse_or(&mut self) -> Result<ComposedFilter> {
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
                Token::Close | Token::At => break,
                t => Err(ArgError::FailedToParse {
                    typ: "query",
                    value: t.to_string().into(),
                    msg: Some("Expected filter or '['".into()),
                })?,
            }
        }

        match &mut exprs[..] {
            [] => Ok(ComposedFilter::default()),
            [e] => Ok(mem::take(e)),
            _ => Ok(ComposedFilter::Or(exprs)),
        }
    }

    fn parse_bracket(&mut self) -> Result<ComposedFilter> {
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

    fn parse_and(&mut self) -> Result<ComposedFilter> {
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
                Token::Close | Token::Or | Token::At => break,
                t => Err(ArgError::FailedToParse {
                    typ: "query",
                    value: t.to_string().into(),
                    msg: Some("Expected filter or '['".into()),
                })?,
            }
        }

        match &mut exprs[..] {
            [] => Ok(ComposedFilter::default()),
            [e] => Ok(mem::take(e)),
            _ => Ok(ComposedFilter::And(exprs)),
        }
    }

    fn parse_filter(&mut self) -> Result<ComposedFilter> {
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

        Ok(ComposedFilter::Filter(f))
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
