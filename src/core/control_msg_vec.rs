use std::{fmt::Display, str::FromStr};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::{AnyControlMsg, Error};

#[derive(Debug, Clone)]
pub struct ControlMsgVec {
    msgs: Vec<AnyControlMsg>,
}

impl ControlMsgVec {
    pub fn new(msgs: Vec<AnyControlMsg>) -> Self {
        Self { msgs }
    }
}

impl Display for ControlMsgVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            shell_words::join(self.msgs.iter().map(|a| a.to_string()))
        )
    }
}

impl FromStr for ControlMsgVec {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(
            shell_words::split(s)?
                .iter()
                .map(|s| s.parse())
                .try_collect()?,
        ))
    }
}

impl Serialize for ControlMsgVec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let res = format!("{self}");
        res.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ControlMsgVec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(|e| {
            serde::de::Error::custom(format!("Invalid value: {e}"))
        })
    }
}

impl IntoIterator for ControlMsgVec {
    type Item = AnyControlMsg;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.msgs.into_iter()
    }
}
