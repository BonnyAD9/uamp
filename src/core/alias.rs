use std::{fmt::Display, str::FromStr};

use pareg::{ArgError, FromArgStr, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Alias {
    pub name: String,
    pub args: Vec<String>,
}

impl FromStr for Alias {
    type Err = ArgError;

    fn from_str(mut s: &str) -> Result<Self> {
        let s0 = s;
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
            args.push(read_arg(&mut s).map_err(|e| {
                e.shift_span(s0.len() - s.len(), s0.to_string())
            })?);
        }

        Ok(Alias { name, args })
    }
}

impl FromArgStr for Alias {}

fn read_arg(s: &mut &str) -> Result<String> {
    let mut res = String::new();
    let mut depth: usize = 0;
    let mut chrs = s.char_indices();

    while let Some((i, c)) = chrs.next() {
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
                    return ArgError::parse_msg(
                        "Additional data after `]`",
                        s.to_string(),
                    )
                    .spanned(i + 1..s.len())
                    .err();
                }
            }
            ',' => {
                if depth == 0 {
                    break;
                }
                res.push(',');
            }
            '/' => {
                res.push('/');
                for (_, c) in &mut chrs {
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
