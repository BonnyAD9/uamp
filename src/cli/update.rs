use pareg::Pareg;
use termal::printcln;

use crate::{
    cli::help::help_update,
    core::{ErrCtx, Error, Result, config::Config},
    env::update,
};

#[derive(Debug, Default)]
pub struct Update {
    force: bool,
    remote: Option<String>,
    manpages: Option<bool>,
    mode: Option<update::Mode>,
    help: bool,
}

impl Update {
    pub(super) fn parse(
        &mut self,
        args: &mut Pareg,
        color: bool,
    ) -> Result<()> {
        while let Some(arg) = args.next() {
            match arg {
                "-h" | "-?" | "--help" => {
                    help_update(color);
                    self.help = true;
                }
                "-f" | "--force" => self.force = true,
                "--remote" => self.remote = Some(args.next_arg()?),
                "--man" => self.manpages = Some(true),
                "--no-man" => self.manpages = Some(false),
                "-m" | "--mode" => self.mode = args.next_arg()?,
                "--" => break,
                _ => args.err_unknown_argument().err()?,
            }
        }

        Ok(())
    }

    pub fn act(&self, conf: &Config) -> Result<()> {
        if self.help {
            return Ok(());
        }

        if !update::ALLOW_SELF_UPDATE && !self.force {
            return Error::InvalidOperation(
                ErrCtx::new(
                    "Uamp self update has been disabled for this build.",
                )
                .into(),
            )
            .hint("Use `--force` to force the update.")
            .err();
        }

        let r = update::try_update(
            self.remote.as_deref().unwrap_or(conf.update_remote()),
            self.mode.as_ref().unwrap_or(conf.update_mode()),
            self.manpages.unwrap_or(update::INSTALL_MANPAGES),
        )?;

        if r {
            printcln!("{'g}Success!{'_} Uamp is updated!");
        } else {
            println!("Uamp is already up to date!");
        }

        Ok(())
    }
}
