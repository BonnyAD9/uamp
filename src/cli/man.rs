use std::io::Write;
use std::process::{Command, Stdio};

use pareg::Pareg;

use crate::core::{Error, Result};

use super::help::help_man;

pub const MAN1: &str = include_str!("../../other/manpages/uamp.1");
pub const MAN5: &str = include_str!("../../other/manpages/uamp.5");

#[derive(Debug)]
pub struct Man {
    pub print: bool,
    pub page: &'static str,
}

impl Man {
    pub(super) fn parse(
        &mut self,
        args: &mut Pareg,
        color: bool,
    ) -> Result<()> {
        while let Some(arg) = args.next() {
            match arg {
                "-h" | "-?" | "--help" => help_man(color),
                "-p" | "--print" => self.print = true,
                "1" | "cli" => self.page = MAN1,
                "5" | "cfg" | "conf" | "config" => self.page = MAN5,
                "--" => break,
                _ => args.err_unknown_argument().err()?,
            }
        }

        Ok(())
    }

    pub fn act(&self) -> Result<()> {
        if self.print {
            println!("{}", self.page);
            return Ok(());
        }

        let mut cmd = Command::new("man")
            .args(["-l", "-"])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| Error::from(e).msg("Failed to start program man."))?;

        let sin = cmd.stdin.as_mut().ok_or_else(|| {
            Error::no_stdin_pipe().msg("Cannot open man page in man.")
        })?;
        write!(sin, "{}", self.page)
            .map_err(|e| Error::from(e).msg("Failed to pipe to man."))?;

        cmd.wait().map_err(|e| {
            Error::from(e).msg("Failed to wait for man to exit.")
        })?;

        Ok(())
    }
}

impl Default for Man {
    fn default() -> Self {
        Self {
            print: false,
            page: MAN1,
        }
    }
}
