use std::{fmt::Display, str::FromStr};

use pareg::{ArgError, FromArgStr};
use serde::{Deserialize, Serialize};

use super::{Error, Result};

#[derive(Clone, Debug)]
pub struct Alias {
    pub name: String,
    pub args: Vec<String>,
}

impl FromStr for Alias {
    type Err = Error;

    fn from_str(mut s: &str) -> Result<Self> {
        let Some((i, _)) = s.char_indices().find(|(_, c)| *c == '[') else {
            return Ok(Self {
                name: s.to_string(),
                args: vec![],
            });
        };

        let name = s[..i].to_string();
        s = &s[i + 1..];

        let mut args = vec![];

        while !s.is_empty() {
            args.push(read_arg(&mut s)?);
        }

        Ok(Alias { name, args })
    }
}

impl FromArgStr for Alias {}

fn read_arg(s: &mut &str) -> Result<String> {
    let mut res = String::new();
    let mut depth: usize = 0;
    let mut chrs = s.chars();

    while let Some(c) = chrs.next() {
        match c {
            '[' => {
                depth += 1;
                res.push('[');
            }
            ']' => {
                if depth == 0 {
                    if chrs.as_str().is_empty() {
                        break;
                    }
                    Err(ArgError::FailedToParse {
                        typ: "Alias",
                        value: chrs.as_str().to_owned().into(),
                        msg: Some("Additional data after ']'".into()),
                    })?;
                }
            }
            ',' => {
                break;
            }
            '/' => {
                res.push('/');
                for c in &mut chrs {
                    res.push(c);
                    if c == '/' {
                        break;
                    }
                }
            }
            _ => res.push(c),
        }
    }

    *s = chrs.as_str();
    Ok(res)
}

impl Display for Alias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.args.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}[{}]", self.name, self.args.join(","))
        }
    }
}

impl Serialize for Alias {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Alias {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(|e| {
            serde::de::Error::custom(format!("Invalid value: {e}"))
        })
    }
}
