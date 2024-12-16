use std::{
    fmt::{Display, Write},
    ops::Range,
};

use pareg::ArgError;

use pareg::Result;

use super::{Filter, SongOrder};

#[derive(Debug)]
pub enum Token {
    Order(SongOrder),
    Filter(Filter),
    And,   // .
    Or,    // +
    Open,  // [
    Close, // ]
    At,    // @
}

pub enum LexerState {
    Filter,
    Order,
}

pub struct Lexer<'a> {
    original: &'a str,
    last_span: Range<usize>,
    data: &'a str,
    pub state: LexerState,
    buf: String,
}

impl<'a> Lexer<'a> {
    const SPECIAL: &'static str = "[].+/@";

    pub fn new(s: &'a str) -> Self {
        Self {
            original: s,
            last_span: 0..0,
            data: s,
            buf: String::new(),
            state: LexerState::Filter,
        }
    }

    pub fn next(&mut self) -> Result<Option<Token>> {
        if self.data.is_empty() {
            self.last_span = self.original.len()..self.original.len();
            return Ok(None);
        }
        self.buf.clear();

        self.last_span.start = self.original.len() - self.data.len();
        let res = match self.state {
            LexerState::Filter => self.next_filter(),
            LexerState::Order => self.next_order(),
        };
        self.last_span.end = self.original.len() - self.data.len();
        res
    }

    pub fn last_span(&self) -> Range<usize> {
        self.last_span.clone()
    }

    pub fn remaining_span(&self) -> Range<usize> {
        self.original.len() - self.data.len()..self.original.len()
    }

    pub fn last_rem_span(&self) -> Range<usize> {
        self.last_span.start..self.original.len()
    }

    pub fn original(&self) -> &'a str {
        self.original
    }

    fn next_order(&mut self) -> Result<Option<Token>> {
        let Some((i, _)) = self.data.char_indices().find(|(_, c)| *c == '@')
        else {
            let res = Token::Order(self.map_err(self.data.parse())?);
            self.consume(self.data.len());
            return Ok(Some(res));
        };

        if i == 0 {
            self.consume(1);
            return Ok(Some(Token::At));
        }

        let res = Token::Order(self.map_err(self.data[..i].parse())?);
        self.consume(i);
        Ok(Some(res))
    }

    fn next_filter(&mut self) -> Result<Option<Token>> {
        let Some((i, c)) = self.find_special() else {
            let res = Token::Filter(self.map_err(self.data.parse())?);
            self.consume(self.data.len());
            return Ok(Some(res));
        };

        if c == '/' {
            // unqoted string
            self.read(i);
            // quoted part of the string
            return self.read_quoted();
        }

        if i != 0 {
            // Unqouted string and no quote
            let data = &self.data[..i];
            let res = Token::Filter(data.parse().map_err(|e: ArgError| {
                e.shift_span(
                    self.last_span.start + data.len(),
                    self.original.to_string(),
                )
            })?);
            self.consume(i);
            return Ok(Some(res));
        }

        // Special char
        self.consume(1);
        let res = match c {
            '.' => Token::And,
            '+' => Token::Or,
            '[' => Token::Open,
            ']' => Token::Close,
            '@' => Token::At,
            _ => unreachable!(),
        };

        Ok(Some(res))
    }

    fn read_quoted(&mut self) -> Result<Option<Token>> {
        let mut err_idxs = vec![];
        self.consume(1);
        err_idxs.push(self.buf.len());

        loop {
            // Find the end of string
            let Some(i) = self.find_char('/') else {
                return ArgError::parse_msg(
                    "Expected ending `/`.",
                    self.original.to_string(),
                )
                .spanned(self.remaining_span())
                .err();
            };

            // Read what was until the end of string
            self.read(i);
            // Consume the ending '/'
            self.consume(1);
            err_idxs.push(self.buf.len());

            // Read outside of the string
            let Some((i, c)) = self.find_special() else {
                self.read(self.data.len());
                break;
            };

            if c != '/' {
                // The string has ended
                self.read(i);
                break;
            }

            if i == 0 {
                // There is "//" inside of the string.
                self.read(1);
                // Continue inside the string
            } else {
                // Read the data outside.
                self.read(i);
                // New string begins
                self.consume(1);
                err_idxs.push(self.buf.len());
                // Read the inside of the string
            }
        }

        let span_adj = |p: usize| {
            self.last_span.start
                + err_idxs
                    .iter()
                    .position(|n| *n >= p)
                    .unwrap_or(err_idxs.len())
        };

        Ok(Some(Token::Filter(self.buf.parse().map_err(
            |e: ArgError| {
                e.map_ctx(|mut c| {
                    c.error_span.start += span_adj(c.error_span.start);
                    c.error_span.end += span_adj(c.error_span.end);
                    c.args[c.error_idx] = self.original.to_string();
                    c
                })
            },
        )?)))
    }

    fn find_special(&self) -> Option<(usize, char)> {
        self.data
            .char_indices()
            .find(|(_, c)| Self::SPECIAL.contains(*c))
    }

    fn find_char(&self, chr: char) -> Option<usize> {
        self.data
            .char_indices()
            .find(|(_, c)| *c == chr)
            .map(|a| a.0)
    }

    #[inline(always)]
    fn read(&mut self, i: usize) {
        self.buf += &self.data[..i];
        self.consume(i);
    }

    #[inline(always)]
    fn consume(&mut self, i: usize) {
        self.data = &self.data[i..];
    }

    fn map_err<T>(&self, e: Result<T>) -> Result<T> {
        e.map_err(|e| {
            e.shift_span(self.last_span.start, self.original.to_string())
        })
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Order(order) => write!(f, "Order({order})"),
            Token::Filter(filter) => write!(f, "Filter({filter})"),
            Token::And => f.write_str("."),
            Token::Or => f.write_str("+"),
            Token::Open => f.write_str("["),
            Token::Close => f.write_str("]"),
            Token::At => f.write_char('@'),
        }
    }
}
