use std::{borrow::Cow, fmt::Display, str::FromStr};

use itertools::Itertools;
use pareg::{ArgErrCtx, ArgError};
use serde::{Deserialize, Serialize};

use super::{AnyControlMsg, Error, Msg, Result};

#[derive(Clone, Debug)]
enum FunctionComponenet {
    Literal(String),
    Variable(usize),
}

#[derive(Clone, Debug)]
enum ControlUnit {
    Simple(AnyControlMsg),
    Composed(Vec<FunctionComponenet>),
}

#[derive(Clone, Debug)]
pub struct ControlFunction {
    args: Vec<String>,
    body: Vec<ControlUnit>,
}

impl ControlFunction {
    pub fn get_msg_vec(&self, args: &[impl AsRef<str>]) -> Result<Vec<Msg>> {
        if args.len() != self.args.len() {
            Error::invalid_operation()
                .msg("Failed to run alias.")
                .reason("Invalid number of arguments to alias.")
                .err()
        } else {
            self.body
                .iter()
                .map(|a| Ok(a.get_msg(args)?.into()))
                .try_collect()
        }
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }
}

impl ControlUnit {
    pub fn get_msg(&self, args: &[impl AsRef<str>]) -> Result<AnyControlMsg> {
        match self {
            ControlUnit::Simple(m) => Ok(m.clone()),
            ControlUnit::Composed(comps) => Self::compose(comps, args),
        }
    }

    fn compose(
        comps: &[FunctionComponenet],
        args: &[impl AsRef<str>],
    ) -> Result<AnyControlMsg> {
        let mut res = String::new();

        for c in comps {
            match c {
                FunctionComponenet::Literal(l) => res += l,
                FunctionComponenet::Variable(v) => res += args[*v].as_ref(),
            }
        }

        Ok(res.parse()?)
    }
}

impl Display for ControlFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.args.is_empty() {
            write!(f, "{{{}}}:", self.args.join(","))?;
        }

        let body = shell_words::join(self.body.iter().map(|a| {
            match a {
                ControlUnit::Simple(s) => s.to_string(),
                ControlUnit::Composed(c) => c
                    .iter()
                    .map(|a| -> Cow<str> {
                        match a {
                            FunctionComponenet::Literal(l) => l.into(),
                            FunctionComponenet::Variable(v) => {
                                format!("${{{}}}", self.args[*v]).into()
                            }
                        }
                    })
                    .join(""),
            }
        }));

        write!(f, "{}", body)
    }
}

impl<T: IntoIterator<Item = AnyControlMsg>> From<T> for ControlFunction {
    fn from(value: T) -> Self {
        Self {
            args: vec![],
            body: value.into_iter().map(ControlUnit::Simple).collect(),
        }
    }
}

impl FromStr for ControlFunction {
    type Err = Error;

    fn from_str(mut s: &str) -> std::result::Result<Self, Self::Err> {
        let s0 = s;

        if !s.starts_with(['[', '{']) {
            let body: Vec<_> = shell_words::split(s)?
                .iter()
                .map(|a| -> Result<_> { Ok(ControlUnit::Simple(a.parse()?)) })
                .try_collect()?;
            return Ok(Self { body, args: vec![] });
        }

        s = &s[1..];
        let Some((i, _)) =
            s.char_indices().find(|(_, c)| matches!(c, ']' | '}'))
        else {
            return ArgError::parse_msg("Missing closing `}`", s0.to_string())
                .spanned(s0.len() - s.len()..s0.len())
                .err()?;
        };

        let args =
            s[..i].split(',').map(|a| a.trim().to_owned()).collect_vec();

        s = &s[i + 1..];
        let Some(s) = s.strip_prefix(':') else {
            return ArgError::parse_msg("Expected `:`", s0.to_string())
                .spanned(s0.len() - s.len()..s0.len() - s.len())
                .err()?;
        };

        let body: Vec<_> = shell_words::split(s)?
            .iter()
            .map(|s| -> Result<ControlUnit> {
                let mut s = s.as_str();
                let mut comps: Vec<FunctionComponenet> = vec![];
                let mut buf = String::new();

                while let Some((i, _)) =
                    s.char_indices().find(|(_, c)| *c == '$')
                {
                    if !s[i + 1..].starts_with('{') {
                        buf += &s[..i + 1];
                        s = &s[i + 1..];
                        continue;
                    }

                    buf += &s[..i];
                    if !buf.is_empty() {
                        comps.push(FunctionComponenet::Literal(buf.clone()));
                        buf.clear();
                    }

                    s = &s[i + 2..];

                    let Some((i, _)) =
                        s.char_indices().find(|(_, c)| *c == '}')
                    else {
                        return ArgError::parse_msg(
                            "Missing closing `}`",
                            s0.to_string(),
                        )
                        .spanned(s0.len() - s.len()..s0.len())
                        .err()?;
                    };

                    let v = &s[..i];
                    s = &s[i + 1..];
                    let Some(p) = args.iter().position(|a| a == v) else {
                        return Err(ArgError::FailedToParse(Box::new(
                            ArgErrCtx::from_msg(
                                "Unknown variable name",
                                s.to_string(),
                            )
                            .spanned(
                                s0.len() - v.len() - s.len()
                                    ..s0.len() - s.len(),
                            ),
                        )))?;
                    };

                    comps.push(FunctionComponenet::Variable(p));
                }

                buf += s;

                if comps.is_empty() {
                    Ok(ControlUnit::Simple(buf.parse()?))
                } else {
                    if !buf.is_empty() {
                        comps.push(FunctionComponenet::Literal(buf.clone()));
                    }

                    Ok(ControlUnit::Composed(comps))
                }
            })
            .try_collect()?;

        Ok(Self { args, body })
    }
}

impl Serialize for ControlFunction {
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

impl<'de> Deserialize<'de> for ControlFunction {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(|e| {
            serde::de::Error::custom(format!("Invalid value: {e}"))
        })
    }
}
