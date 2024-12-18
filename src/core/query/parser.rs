use std::mem;

use pareg::ArgError;

use super::{
    lexer::{Lexer, LexerState, Token},
    ComposedFilter, Query, SongOrder,
};

pub struct Parser<'a> {
    lex: Lexer<'a>,
    cur: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn parse_composed_filter(
        s: &'a str,
    ) -> Result<ComposedFilter, ArgError> {
        Self {
            lex: Lexer::new(s),
            cur: None,
        }
        .parse_composed_filter_inner()
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
        let filter = self.parse_comp_filter()?;
        if !matches!(self.cur()?, Some(Token::At)) {
            return Ok(Query::new(filter, None));
        }

        self.lex.state = LexerState::Order;
        self.next()?;

        if matches!(self.cur()?, Some(Token::At) | None) {
            return Ok(Query::new(filter, None));
        }

        let order = self.parse_order()?;

        match self.cur()? {
            Some(Token::At) | None => {}
            Some(_) => {
                return self
                    .err_unused_input()
                    .hint("Maybe you are missing `@`.")
                    .err();
            }
        }

        self.lex.state = LexerState::Filter;

        Ok(Query::new(filter, Some(order)))
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
                        .inline_msg("Expected filter or `[`.")
                        .err()
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
                        .inline_msg("Expected `+` or `]`.")
                        .err()
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
                .inline_msg("Expected `]`.")
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
                        .inline_msg("Expected filter or `[`.")
                        .err()
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
                        .inline_msg("Expected `.`, `]`, `+` or `@`.")
                        .err()
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
                    .err()
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
        ArgError::parse_msg(
            format!(
                "Unexpected token `{}`",
                &self.lex.original()[self.lex.last_span()]
            ),
            self.lex.original().to_string(),
        )
        .spanned(self.lex.last_span())
    }

    fn err_unused_input(&self) -> ArgError {
        ArgError::parse_msg("Unused input.", self.lex.original().to_string())
            .hint("This input is unused and can be removed.")
            .spanned(self.lex.last_rem_span())
    }
}
