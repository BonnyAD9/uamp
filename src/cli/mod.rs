mod action;
mod args;
mod err;
mod help;
mod macros;
mod parsable;
mod parsers;

pub use self::{action::*, args::*, err::{Error as CliError, Result as CliResult}};
pub use args::*;

///! Contains tools for the cli, mostly parsing

/// Contains auto generated parsing functions
pub mod parse {
    /// Parses single argument to control message (e.g. if the arguments were
    /// `uamp i pp=play`, this would be able to parse only the `pp=play`)
    pub use super::parsers::parse_control_message;
}
