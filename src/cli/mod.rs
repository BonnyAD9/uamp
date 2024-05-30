mod action;
mod args;
mod help;

pub use self::{action::*, args::*};
/*pub use args::*;

///! Contains tools for the cli, mostly parsing

/// Contains auto generated parsing functions
pub mod parse {
    /// Parses single argument to control message (e.g. if the arguments were
    /// `uamp i pp=play`, this would be able to parse only the `pp=play`)
    pub use super::parsers::parse_control_message;
}*/
