use pareg::{ArgError, FromArg, starts_any, val_arg};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum UpdateMode {
    #[default]
    LatestTag,
    LatestCommit,
    Branch(String),
}

impl<'a> FromArg<'a> for UpdateMode {
    fn from_arg(arg: &'a str) -> pareg::Result<Self> {
        match arg {
            "tag" | "latest-tag" | "LatestTag" => Ok(Self::LatestTag),
            "commit" | "latest-commit" | "LatestCommit" => {
                Ok(Self::LatestCommit)
            }
            v if starts_any!(v, "branch=", "Branch=") => {
                Ok(Self::Branch(val_arg(arg, '=')?))
            }
            _ => ArgError::invalid_value("Invalid update mode.", arg)
                .hint("Valid options are: `tag`, `commit`, `branch=<branch>`.")
                .err(),
        }
    }
}
