use std::{
    env,
    process::{Command, Stdio},
};

use log::{info, warn};
use pareg::{ArgError, ArgIterator, ByRef};

use crate::{
    background_app::run_background_app,
    config::Config,
    core::{Error, Result},
};

use super::help::help_run;

#[derive(Default)]
pub struct Run {
    pub detach: bool,

    pub port: Option<u16>,
    pub server_address: Option<String>,
}

impl Run {
    pub(super) fn parse<'a, I>(
        &mut self,
        args: &mut ArgIterator<'a, I>,
    ) -> Result<()>
    where
        I: Iterator,
        I::Item: ByRef<&'a str>,
    {
        while let Some(arg) = args.next() {
            match arg {
                "-h" | "-?" | "--help" => help_run(),
                "-d" | "--detach" => self.detach = true,
                "-p" | "--port" => self.port = args.next_arg()?,
                "-a" | "--address" => self.server_address = args.next_arg()?,
                _ => Err(ArgError::UnknownArgument(arg.into()))?,
            }
        }
        Ok(())
    }

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

    pub fn run_app(self, mut conf: Config) -> Result<()> {
        if self.detach {
            warn!("Detach is set to detached when not running as detached.");
        }

        self.update_config(&mut conf);
        run_background_app(conf)
    }

    pub fn run_detached(self) -> Result<()> {
        if !self.detach {
            warn!("Detached is not set to detached when running as detached");
        }

        let cmd = env::args_os().next().ok_or(Error::NoProgramName)?;
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

        let child = cmd.spawn()?;
        let id = child.id();
        println!("Spawned detached process with id {id}");
        info!("Spawned detached process with id {id}");

        Ok(())
    }
}
