use std::{
    env,
    process::{Command, Stdio},
};

use log::{info, warn};
use pareg::Pareg;

use crate::{
    background_app::run_background_app,
    core::{AnyControlMsg, Error, Result, config::Config},
};

use super::{help::help_run, port::Port};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Describes how to start a new uamp instance.
#[derive(Default, Debug)]
pub struct Run {
    /// True if the new instance should run as a detached process.
    pub detach: bool,

    /// Port which should be used for the new instance. If this sit set, the
    /// new instance will have disabled saves of configuration.
    pub port: Option<u16>,
    /// Server address to be used in the new instance. If this is set, the new
    /// instance will have disabled saves of configuration.
    pub server_address: Option<String>,

    /// Messages that will be performed after the initialization of the new
    /// instance.
    pub init: Vec<AnyControlMsg>,
}

impl Run {
    /// Parses the run arguments.
    ///
    /// # Errors
    /// - The arguments are invalid.
    pub(super) fn parse(
        &mut self,
        args: &mut Pareg,
        color: bool,
    ) -> Result<()> {
        while let Some(arg) = args.next() {
            match arg {
                "-h" | "-?" | "--help" => help_run(color),
                "-d" | "--detach" => self.detach = true,
                "-p" | "--port" => {
                    self.port = Some(args.next_arg::<Port>()?.0)
                }
                "-a" | "--address" => {
                    self.server_address = Some(args.next_arg()?)
                }
                _ => self.init.push(args.cur_arg()?),
            }
        }
        Ok(())
    }

    /// Runs the app NOT IN DETACHED MODE. The value of [`Self::detach`] is
    /// ignored.
    ///
    /// # Errors
    /// - The app fails to start.
    pub fn run_app(self, mut conf: Config) -> Result<()> {
        if self.detach {
            warn!("Detach is set to detached when not running as detached.");
        }

        self.update_config(&mut conf);
        run_background_app(conf, self.init)
    }

    /// Runs the app IN DETACHED MODE. The value of [`Self::detach`] is
    /// ignored.
    ///
    /// # Errors
    /// - The command fails to spawn a new process.
    pub fn run_detached(self) -> Result<()> {
        if !self.detach {
            warn!("Detached is not set to detached when running as detached");
        }

        let cmd = env::args_os().next().ok_or(
            Error::no_program_name().msg("Failed to run detached uamp."),
        )?;
        let mut cmd = Command::new(cmd);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());

        cmd.arg("run");

        if let Some(p) = self.port {
            cmd.args(["-p", &p.to_string()]);
        }
        if let Some(a) = self.server_address {
            cmd.args(["-a", &a]);
        }

        cmd.args(self.init.into_iter().map(|a| a.to_string()));

        let child = cmd
            .spawn()
            .map_err(|e| Error::io(e).msg("Failed to spawn detached uamp."))?;
        let id = child.id();
        println!("Spawned detached process with id {id}");
        info!("Spawned detached process with id {id}");

        Ok(())
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl Run {
    fn update_config(&self, conf: &mut Config) {
        if let Some(v) = self.port {
            conf.port_set(v);
        }
        if let Some(v) = &self.server_address {
            *conf.server_address_mut() = v.to_owned();
        }

        if conf.changed() {
            conf.config_path = None;
        }
    }
}
