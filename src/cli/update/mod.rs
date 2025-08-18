use pareg::Pareg;
use termal::printcln;

use crate::{
    cli::{help::help_update, update::action::Action},
    core::{ErrCtx, Error, Result, config::Config},
    env::update,
};

mod action;

#[derive(Debug, Default)]
pub struct Update {
    force: bool,
    no_do: bool,
    remote: Option<String>,
    manpages: Option<bool>,
    mode: Option<update::Mode>,
    action: Action,
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
                    self.no_do = true;
                }
                "-f" | "--force" => self.force = true,
                "--remote" => self.remote = Some(args.next_arg()?),
                "--man" => self.manpages = Some(true),
                "--no-man" => self.manpages = Some(false),
                "-m" | "--mode" => self.mode = args.next_arg()?,
                "--enabled" => self.action = Action::CheckEnabled,
                "--" => break,
                _ => args.err_unknown_argument().err()?,
            }
        }

        Ok(())
    }

    pub fn act(&self, conf: &Config) -> Result<()> {
        if self.no_do {
            return Ok(());
        }

        match self.action {
            Action::CheckEnabled => self.act_enabled(),
            Action::Update => self.act_update(conf),
        }
    }

    fn act_enabled(&self) -> Result<()> {
        if update::ALLOW_SELF_UPDATE {
            println!("yes");
        } else {
            println!("no");
        }

        Ok(())
    }

    fn act_update(&self, conf: &Config) -> Result<()> {
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
