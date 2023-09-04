use std::str::FromStr;

use itertools::Itertools;

use crate::{
    cli::{parse::parse_control_message, Error as CliError},
    core::msg::{get_control_string, ControlMsg},
};

#[derive(Clone)]
pub struct Action {
    pub controls: Vec<ControlMsg>,
}

impl Action {
    pub fn join(&mut self, mut other: Action) {
        self.controls.append(&mut other.controls);
    }
}

impl ToString for Action {
    fn to_string(&self) -> String {
        self.controls
            .iter()
            .map(|c| get_control_string(c))
            .join(" ")
    }
}

impl FromStr for Action {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = s
            .split(char::is_whitespace)
            .filter(|s| !s.is_empty())
            .map(|s| parse_control_message(s))
            .try_collect()?;
        Ok(Self { controls: res })
    }
}
