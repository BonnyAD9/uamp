use pareg::Pareg;

use crate::{core::Result, env::install::install};

#[derive(Debug)]
pub struct Install {
    root: String,
    exe: String,
    man: bool,
}

impl Install {
    pub fn parse(args: &mut Pareg) -> Result<Self> {
        let mut res = Self::default();

        while let Some(arg) = args.next() {
            match arg {
                "--man" => res.man = args.next_arg()?,
                "--root" => res.root = args.next_arg()?,
                "--exe" => res.exe = args.next_arg()?,
                _ => args.err_unknown_argument().err()?,
            }
        }

        Ok(res)
    }

    pub fn act(&self) -> Result<()> {
        install(self.root.as_ref(), self.exe.as_ref(), self.man)
    }
}

impl Default for Install {
    fn default() -> Self {
        Self {
            root: "/".to_string(),
            exe: "/usr/bin/uamp".to_string(),
            man: true,
        }
    }
}
