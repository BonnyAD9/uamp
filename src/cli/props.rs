use std::io::{self, IsTerminal};

use super::printers::PrintStyle;

#[derive(Debug, Clone)]
pub struct Props {
    /// Determines whether color should be used in standard output.
    pub color: bool,
    /// Determines the style of output for stdout.
    pub print_style: PrintStyle,
}

impl Default for Props {
    fn default() -> Self {
        Self {
            color: io::stdout().is_terminal(),
            print_style: Default::default(),
        }
    }
}
