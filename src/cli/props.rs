use std::{
    borrow::Cow,
    io::{self, IsTerminal},
};

use super::printers::PrintStyle;

#[derive(Debug, Clone)]
pub struct Props {
    /// Determines whether color should be used in standard output.
    pub color: bool,
    /// Determines the style of output for stdout.
    pub print_style: PrintStyle,
    /// Verbosity. 0 is default, negative is less, positive is more.
    pub verbosity: i32,
}

impl Props {
    pub fn with_verbosity(&self, v: Option<i32>) -> Cow<'_, Self> {
        if let Some(v) = v {
            let mut res = self.clone();
            res.verbosity = v;
            Cow::Owned(res)
        } else {
            Cow::Borrowed(self)
        }
    }
}

impl Default for Props {
    fn default() -> Self {
        Self {
            color: io::stdout().is_terminal(),
            print_style: Default::default(),
            verbosity: 0,
        }
    }
}
