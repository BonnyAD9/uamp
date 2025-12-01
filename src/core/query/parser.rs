use std::mem;

use pareg::ArgError;

use crate::core::query::Base;

use super::{
    ComposedFilter, Query, SongOrder,
    lexer::{Lexer, LexerState, Token},
};

pub struct Parser<'a> {
    lex: Lexer<'a>,
    cur: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn parse_composed_filter(
        s: &'a str,
    ) -> Result<ComposedFilter, ArgError> {
        let mut lex = Lexer::new(s);
        lex.state = LexerState::Filter;
        Self { lex, cur: None }.parse_composed_filter_inner()
    }

    pub fn parse_query(s: &'a str) -> Result<Query, ArgError> {
        Self {
            lex: Lexer::new(s),
            cur: None,
        }
        .parse_query_inner()
    }

    fn parse_composed_filter_inner(
        &mut self,
    ) -> Result<ComposedFilter, ArgError> {
        let r = self.parse_or()?;
        if self.cur()?.is_some() {
            Err(self.err_unused_input())?;
        }
        Ok(r)
    }

    fn parse_query_inner(&mut self) -> Result<Query, ArgError> {
        let r = self.parse_q()?;
        if self.cur()?.is_some() {
            Err(self.err_unused_input())?;
        }
        Ok(r)
    }

    fn parse_q(&mut self) -> Result<Query, ArgError> {
        let bases = self.parse_bases()?;

        let filter = self.parse_comp_filter()?;
        if !matches!(self.cur()?, Some(Token::At)) {
            return Ok(Query::new(bases, filter, None, None));
        }

        self.lex.state = LexerState::Order;
        self.next()?;

        let order = if !matches!(self.cur()?, Some(Token::At) | None) {
            Some(self.parse_order()?)
        } else {
            None
        };

        if !matches!(self.cur()?, Some(Token::At) | None) {
            return self
                .err_unused_input()
                .hint("Maybe you are missing `@`.")
                .err();
        }

        self.lex.state = LexerState::Unique;

        self.next()?;

        if self.cur()?.is_none() {
            return Ok(Query::new(bases, filter, order, None));
        }

        let Some(Token::Unique(unique)) = self.cur.take() else {
            return self
                .err_unexpected_token()
                .hint("Expected unique specifier.")
                .err();
        };

        if self.next()?.is_some() {
            return self.err_unused_input().err();
        }

        Ok(Query::new(bases, filter, order, Some(unique)))
    }

    fn parse_bases(&mut self) -> Result<Vec<Base>, ArgError> {
        self.cur()?;
        if !matches!(self.cur, Some(Token::Comma)) {
            self.lex.state = LexerState::Filter;
            return Ok(vec![]);
        }

        self.lex.state = LexerState::Bases;
        let mut res = vec![];

        loop {
            self.cur()?;
            match self.cur.take() {
                Some(Token::Comma) => continue,
                Some(Token::Base(b)) => res.push(b),
                Some(Token::At) | None => {
                    self.lex.state = LexerState::Filter;
                    break;
                }
                _ => {
                    return self
                        .err_unexpected_token()
                        .inline_msg("Expected base.")
                        .err();
                }
            }
        }

        Ok(res)
    }

    fn parse_order(&mut self) -> Result<SongOrder, ArgError> {
        self.cur()?;
        match self.cur.take() {
            Some(Token::Order(o)) => Ok(o),
            _ => self
                .err_unexpected_token()
                .inline_msg("Expected song order.")
                .err(),
        }
    }

    fn parse_comp_filter(&mut self) -> Result<ComposedFilter, ArgError> {
        if matches!(self.cur()?, Some(Token::At)) {
            Ok(Default::default())
        } else {
            self.parse_or()
        }
    }

    fn parse_or(&mut self) -> Result<ComposedFilter, ArgError> {
        let mut exprs = vec![];

        while let Some(t) = self.cur()? {
            match t {
                Token::Filter(_) => exprs.push(self.parse_and()?),
                Token::Open => {
                    exprs.push(self.parse_bracket()?);
                }
                _ => {
                    return self
                        .err_unexpected_token()
                        .inline_msg("Expected filter or `{`.")
                        .err();
                }
            }

            let Some(t) = self.cur()? else {
                break;
            };

            match t {
                Token::Or => _ = self.next()?,
                Token::Close | Token::At => break,
                _ => {
                    return self
                        .err_unexpected_token()
                        .inline_msg("Expected `+` or `}`.")
                        .err();
                }
            }
        }

        match &mut exprs[..] {
            [] => Ok(ComposedFilter::default()),
            [e] => Ok(mem::take(e)),
            _ => Ok(ComposedFilter::Or(exprs)),
        }
    }

    fn parse_bracket(&mut self) -> Result<ComposedFilter, ArgError> {
        self.next()?;
        let r = self.parse_and()?;
        if !matches!(self.cur()?, Some(Token::Close)) {
            return self
                .err_unexpected_token()
                .inline_msg("Expected `}`.")
                .err();
        }
        self.next()?;
        Ok(r)
    }

    fn parse_and(&mut self) -> Result<ComposedFilter, ArgError> {
        let mut exprs = vec![];

        while let Some(t) = self.cur()? {
            match t {
                Token::Filter(_) => exprs.push(self.parse_filter()?),
                Token::Open => {
                    exprs.push(self.parse_bracket()?);
                }
                _ => {
                    return self
                        .err_unexpected_token()
                        .inline_msg("Expected filter or `{`.")
                        .err();
                }
            }

            let Some(t) = self.cur()? else {
                break;
            };

            match t {
                Token::And => _ = self.next()?,
                Token::Close | Token::Or | Token::At => break,
                _ => {
                    return self
                        .err_unexpected_token()
                        .inline_msg("Expected `.`, `}`, `+` or `@`.")
                        .err();
                }
            }
        }

        match &mut exprs[..] {
            [] => Ok(ComposedFilter::default()),
            [e] => Ok(mem::take(e)),
            _ => Ok(ComposedFilter::And(exprs)),
        }
    }

    fn parse_filter(&mut self) -> Result<ComposedFilter, ArgError> {
        self.cur()?;
        let f = match self.cur.take() {
            Some(Token::Filter(f)) => f,
            _ => {
                return self
                    .err_unexpected_token()
                    .inline_msg("Expected filter.")
                    .err();
            }
        };

        Ok(ComposedFilter::Filter(f))
    }

    fn next(&mut self) -> Result<Option<&Token>, ArgError> {
        self.cur = self.lex.next()?;
        Ok(self.cur.as_ref())
    }

    fn cur(&mut self) -> Result<Option<&Token>, ArgError> {
        if let Some(ref c) = self.cur {
            Ok(Some(c))
        } else {
            self.next()
        }
    }

    fn err_unexpected_token(&self) -> ArgError {
        ArgError::failed_to_parse(
            format!(
                "Unexpected token `{}`",
                &self.lex.original()[self.lex.last_span()]
            ),
            self.lex.original(),
        )
        .spanned(self.lex.last_span())
    }

    fn err_unused_input(&self) -> ArgError {
        ArgError::failed_to_parse("Unused input.", self.lex.original())
            .hint("This input is unused and can be removed.")
            .spanned(self.lex.last_rem_span())
    }
}
