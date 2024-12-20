use pareg::Pareg;

mod tab_complete;

use crate::core::Result;

pub use self::tab_complete::*;

use super::help::help_internal;

#[derive(Debug)]
pub enum Internal {
    None,
    TabComplete(TabComplete),
}

impl Internal {
    pub fn new(args: &mut Pareg, color: bool) -> Result<Self> {
        let Some(a) = args.next() else {
            return args.err_no_more_arguments().err()?;
        };

        match a {
            "-h" | "-?" | "--help" => {
                help_internal(color);
                Ok(Self::None)
            }
            "tab-complete" => Ok(Self::TabComplete(TabComplete::new()?)),
            _ => args.err_unknown_argument().err()?,
        }
    }

    pub fn act(&self) -> Result<()> {
        match self {
            Self::None => Ok(()),
            Self::TabComplete(t) => t.act(),
        }
    }
}
