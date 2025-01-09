use std::str::FromStr;

use pareg::{ArgError, ArgInto, FromArgStr};

use crate::core::config;

#[derive(Clone, Copy, Debug)]
pub struct Port(pub u16);

impl FromStr for Port {
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" | "default" => Ok(Self(config::DEFAULT_PORT)),
            "debug" => Ok(Self(config::DEBUG_PORT)),
            "release" | "uamp" => Ok(Self(config::RELEASE_PORT)),
            _ => Ok(Self(s.arg_into().map_err(|e| {
                e.hint(
                    "Port may be port number, `default`, `debug` or `release`",
                )
            })?)),
        }
    }
}

impl FromArgStr for Port {}
