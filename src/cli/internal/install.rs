use pareg::Pareg;

use crate::{
    core::Result,
    env::install::{self, install},
};

#[derive(Debug)]
pub struct Install {
    root: Option<String>,
    exe: String,
    man: bool,
}

impl Install {
    pub fn parse(args: &mut Pareg) -> Result<Self> {
        let mut res = Self::default();

        while let Some(arg) = args.next() {
            match arg {
                "--man" => res.man = args.next_arg()?,
                "--root" => res.root = Some(args.next_arg()?),
                "--exe" => res.exe = args.next_arg()?,
                _ => args.err_unknown_argument().err()?,
            }
        }

        Ok(res)
    }

    pub fn act(&self) -> Result<()> {
        install(
            self.root.as_ref().map(|a| a.as_ref()),
            self.exe.as_ref(),
            self.man,
        )
    }
}

impl Default for Install {
    fn default() -> Self {
        Self {
            root: None,
            exe: "/usr/bin/uamp".to_string(),
            man: install::INSTALL_MANPAGES,
        }
    }
}
