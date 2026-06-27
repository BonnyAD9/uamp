use std::{
    fmt::{Display, Write},
    sync::Arc,
};

use pareg::{FromArg, key_val_arg};
use serde::{Deserialize, Serialize};

use crate::core::query::Query;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTag {
    pub hidden: Option<bool>,
    pub name: Arc<str>,
    pub query: Query,
}

impl Display for AddTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.hidden {
            Some(true) => f.write_char('-')?,
            Some(false) => f.write_char('+')?,
            _ => {}
        }

        write!(f, "{}:{}", self.name, self.query)
    }
}

impl<'a> FromArg<'a> for AddTag {
    fn from_arg(arg: &'a str) -> pareg::Result<Self> {
        let (mut name, query) = key_val_arg::<&str, _>(arg, ':')?;
        let mut hidden = None;
        if let Some(n) = name.strip_prefix('-') {
            hidden = Some(true);
            name = n;
        } else if let Some(n) = name.strip_prefix('+') {
            hidden = Some(false);
            name = n
        }
        Ok(Self {
            hidden,
            name: name.into(),
            query,
        })
    }
}
