use std::fmt::Display;

use pareg::ArgError;

use crate::core::Result;

use super::Filter;

pub enum Token {
    Filter(Filter),
    And,   // .
    Or,    // +
    Open,  // [
    Close, // ]
}

pub struct Lexer<'a> {
    data: &'a str,
    buf: String,
}

impl<'a> Lexer<'a> {
    const SPECIAL: &'static str = "[].+/";

    pub fn new(s: &'a str) -> Self {
        Self {
            data: s,
            buf: String::new(),
        }
    }

    pub fn next(&mut self) -> Result<Option<Token>> {
        if self.data.is_empty() {
            return Ok(None);
        }
        self.buf.clear();

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
            let res = Token::Filter(self.data[..i].parse()?);
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
            Token::Filter(filter) => write!(f, "Filter({filter:?})"),
            Token::And => f.write_str("."),
            Token::Or => f.write_str("+"),
            Token::Open => f.write_str("["),
            Token::Close => f.write_str("]"),
        }
    }
}
