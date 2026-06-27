use std::{fmt::Display, sync::Arc};

use pareg::{FromArg, key_val_arg};
use serde::{Deserialize, Serialize};

use crate::core::query::Query;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveTag {
    pub name: Arc<str>,
    pub query: Query,
}

impl Display for RemoveTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.name, self.query)
    }
}

impl<'a> FromArg<'a> for RemoveTag {
    fn from_arg(arg: &'a str) -> pareg::Result<Self> {
        let (name, query) = key_val_arg(arg, ':')?;
        Ok(Self { name, query })
    }
}
