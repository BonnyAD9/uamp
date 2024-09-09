use std::fmt::{Display, Write};

use pareg::ArgError;

use crate::core::Result;

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
    data: &'a str,
    pub state: LexerState,
    buf: String,
}

impl<'a> Lexer<'a> {
    const SPECIAL: &'static str = "[].+/@";

    pub fn new(s: &'a str) -> Self {
        Self {
            data: s,
            buf: String::new(),
            state: LexerState::Filter,
        }
    }

    pub fn next(&mut self) -> Result<Option<Token>> {
        if self.data.is_empty() {
            return Ok(None);
        }
        self.buf.clear();

        match self.state {
            LexerState::Filter => self.next_filter(),
            LexerState::Order => self.next_order(),
        }
    }

    fn next_order(&mut self) -> Result<Option<Token>> {
        let Some((i, _)) = self.data.char_indices().find(|(_, c)| *c == '@')
        else {
            let res = Token::Order(self.data.parse()?);
            self.consume(self.data.len());
            return Ok(Some(res));
        };

        if i == 0 {
            self.consume(1);
            return Ok(Some(Token::At));
        }

        let res = Token::Order(self.data[..i].parse()?);
        self.consume(i);
        Ok(Some(res))
    }

    fn next_filter(&mut self) -> Result<Option<Token>> {
        let Some((i, c)) = self.find_special() else {
            let res = Token::Filter(self.data.parse()?);
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
            let res = Token::Filter(data.parse()?);
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
        self.consume(1);

        loop {
            // Find the end of string
            let Some(i) = self.find_char('/') else {
                Err(ArgError::FailedToParse {
                    typ: "Query",
                    value: self.data.to_owned().into(),
                    msg: Some("Expected ending `/`".into()),
                })?
            };

            // Read what was until the end of string
            self.read(i);
            // Consume the ending '/'
            self.consume(1);

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
                // Read the inside of the string
            }
        }

        Ok(Some(Token::Filter(self.buf.parse()?)))
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
