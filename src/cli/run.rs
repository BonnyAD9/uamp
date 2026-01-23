use std::{
    env,
    path::Path,
    process::{Child, Command, Stdio},
};

use log::info;
use pareg::Pareg;

use crate::{
    background_app::run_background_app,
    core::{AnyControlMsg, Error, Result, config::Config},
    env::RunType,
    web_client::run_web_client,
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

    pub run_type: RunType,

    /// Port which should be used for the new instance. If this sit set, the
    /// new instance will have disabled saves of configuration.
    pub port: Option<u16>,
    /// Server address to be used in the new instance. If this is set, the new
    /// instance will have disabled saves of configuration.
    pub server_address: Option<String>,

    /// Messages that will be performed after the initialization of the new
    /// instance.
    pub init: Vec<AnyControlMsg>,

    pub run: Option<bool>,

    pub config: Option<String>,
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
                "-h" | "-?" | "--help" => {
                    help_run(color);
                    self.run = Some(self.run.unwrap_or_default());
                }
                "-d" | "--detach" => {
                    self.detach = true;
                    self.run = Some(true);
                }
                "-p" | "--port" => {
                    self.port = Some(args.next_arg::<Port>()?.0);
                    self.run = Some(true);
                }
                "-a" | "--address" => {
                    self.server_address = Some(args.next_arg()?);
                    self.run = Some(true);
                }
                "-b" | "--background" => {
                    self.run_type = RunType::Background;
                }
                "-w" | "--web" => {
                    self.run_type = RunType::WebClient;
                }
                "--config" => {
                    self.config = Some(args.next_arg()?);
                }
                _ => {
                    self.init.push(args.cur_arg()?);
                    self.run = Some(true);
                }
            }
        }
        Ok(())
    }

    pub fn blocking(&self) -> bool {
        !self.detach && matches!(self.run_type, RunType::Background)
    }

    pub fn run_app(self, mut conf: Config) -> Result<()> {
        self.update_config(&mut conf);
        match self.run_type {
            RunType::Background => {
                run_background_app(conf, self.init)?;
            }
            RunType::WebClient => {
                run_web_client(&conf, self.config, self.init)?;
            }
        }

        Ok(())
    }

    /// Runs the app IN DETACHED MODE. The value of [`Self::detach`] is
    /// ignored.
    ///
    /// # Errors
    /// - The command fails to spawn a new process.
    pub fn run_detached(self, conf: &Config) -> Result<()> {
        match self.run_type {
            RunType::Background => {
                run_detached(
                    &self.init,
                    self.config,
                    self.server_address.as_deref(),
                    self.port,
                )?;
            }
            RunType::WebClient => {
                // TODO: avoid cloning
                let mut conf = conf.clone();
                self.update_config(&mut conf);
                run_web_client(&conf, self.config, self.init)?;
            }
        }

        Ok(())
    }
}

pub fn run_detached(
    init: &[AnyControlMsg],
    config: Option<impl AsRef<Path>>,
    adr: Option<&str>,
    port: Option<u16>,
) -> Result<Child> {
    let cmd = env::current_exe()
        .map_err(|e| Error::from(e).msg("Failed to run detached uamp."))?;
    let mut cmd = Command::new(cmd);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    cmd.arg("run");

    if let Some(config) = config {
        cmd.arg("--config");
        cmd.arg(config.as_ref());
    }
    if let Some(adr) = adr {
        cmd.args(["-a", adr]);
    }
    if let Some(port) = port {
        cmd.args(["-p", &port.to_string()]);
    }

    cmd.args(init.iter().map(|a| a.to_string()));

    let child = cmd
        .spawn()
        .map_err(|e| Error::io(e).msg("Failed to spawn detached uamp."))?;
    let id = child.id();
    println!("Spawned detached process with id {id}");
    info!("Spawned detached process with id {id}");

    Ok(child)
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl Run {
    fn update_config(&self, conf: &mut Config) {
        let mut change = false;
        if let Some(v) = self.port {
            conf.set_port(v);
            change = true;
        }
        if let Some(v) = &self.server_address {
            *conf.mut_server_address() = v.to_owned();
            change = true;
        }

        if change {
            conf.config_path = None;
        }
    }
}
