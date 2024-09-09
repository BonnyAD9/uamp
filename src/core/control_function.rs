use std::{fmt::Display, str::FromStr};

use itertools::Itertools;
use pareg::ArgError;
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
            Err(Error::InvalidOperation(
                "Invalid number of arguments to alias.",
            ))
        } else {
            self.body
                .iter()
                .map(|a| Ok(a.get_msg(args)?.into()))
                .try_collect()
        }
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

        res.parse()
    }
}

impl Display for ControlFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]:{}",
            self.args.join(","),
            shell_words::join(self.body.iter().map(|a| {
                match a {
                    ControlUnit::Simple(s) => s.to_string(),
                    ControlUnit::Composed(c) => c
                        .iter()
                        .map(|a| match a {
                            FunctionComponenet::Literal(l) => l,
                            FunctionComponenet::Variable(v) => &self.args[*v],
                        })
                        .join(""),
                }
            }))
        )
    }
}

impl FromStr for ControlFunction {
    type Err = Error;

    fn from_str(mut s: &str) -> std::result::Result<Self, Self::Err> {
        if !s.starts_with('[') {
            let body: Vec<_> = shell_words::split(s)?
                .iter()
                .map(|a| -> Result<_> { Ok(ControlUnit::Simple(a.parse()?)) })
                .try_collect()?;
            return Ok(Self { body, args: vec![] });
        }

        s = &s[1..];
        let Some((i, _)) = s.char_indices().find(|(_, c)| *c == ']') else {
            return Err(ArgError::FailedToParse {
                typ: "ControlFunction",
                value: s.to_owned().into(),
                msg: Some("Missing closing ']'".into()),
            }
            .into());
        };

        let args =
            s[..i].split(',').map(|a| a.trim().to_owned()).collect_vec();

        s = &s[i + 1..];

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
                        return Err(ArgError::FailedToParse {
                            typ: "ControlFunction",
                            value: s.to_owned().into(),
                            msg: Some("Missing closing '}'".into()),
                        }
                        .into());
                    };

                    let v = &s[..i];
                    s = &s[i + 1..];
                    let Some(p) = args.iter().position(|a| a == v) else {
                        return Err(ArgError::FailedToParse {
                            typ: "ControlFunction",
                            value: v.to_owned().into(),
                            msg: Some("Unknown variable name".into()),
                        }
                        .into());
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
