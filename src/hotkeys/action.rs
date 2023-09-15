use std::str::FromStr;

use itertools::Itertools;

use crate::{
    cli::{parse::parse_control_message, CliError},
    core::msg::{get_control_string, ControlMsg},
};

/// Contains action that produces sed of control messages
#[derive(Clone)]
pub struct Action {
    pub controls: Vec<ControlMsg>,
}

impl Action {
    /// Appends this action with the other action
    pub(super) fn join(&mut self, mut other: Action) {
        self.controls.append(&mut other.controls);
    }

    /// Remuves the given part of this action
    pub(super) fn strip(&mut self, other: &Action) {
        let mut i = 0;

        'outer: while self.controls.len() - i >= other.controls.len() {
            let mut j = 0;
            while j < other.controls.len() {
                if self.controls[j + i] != other.controls[j + i] {
                    i += 1;
                    continue 'outer;
                }
                j += 1;
            }

            self.controls.drain(i..i + other.controls.len());
            return;
        }
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
