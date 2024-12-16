use std::io::{self, IsTerminal};

use pareg::{has_any_key, FromArg, Pareg};

use crate::{
    cli::help::help_version,
    core::{config::Config, Error, Result},
};

use super::{
    help::{help, help_all},
    Action, Instance, Run,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Contains the CLI arguments values.
#[derive(Debug)]
pub struct Args {
    /// Actions to do.
    pub actions: Vec<Action>,

    /// Port for the server.
    pub port: Option<u16>,
    /// Address of the server.
    pub server_address: Option<String>,

    /// The mailoop should not run, unless `must_run` is set to `true`.
    pub should_exit: bool,
    /// The mainloop should run in all cases if this is `true`.
    pub run: Option<Run>,

    /// Determines whether color should be used in standard output.
    pub stdout_color: bool,
}

impl Args {
    /// Parses the arguments.
    ///
    /// # Returns
    /// The parsed arguments.
    ///
    /// # Errors
    /// - The arguments are invalid.
    pub fn parse(mut args: Pareg) -> Result<Self> {
        let mut res = Args::default();

        res.top_level(&mut args)?;

        Ok(res)
    }

    /// Loads config based on the arguments.
    ///
    /// # Returns
    /// New configuration readed from the default file and modified by the
    /// values in the arguments.
    pub fn make_config(&self) -> Config {
        let mut res = Config::from_default_json();

        if let Some(v) = self.port {
            res.port_set(v);
        }
        if let Some(v) = &self.server_address {
            *res.server_address_mut() = v.to_owned();
        }

        if res.changed() {
            res.config_path = None;
        }

        res
    }
}

impl Default for Args {
    fn default() -> Self {
        Self {
            actions: vec![],
            port: None,
            server_address: None,
            should_exit: false,
            run: None,
            stdout_color: io::stdout().is_terminal(),
        }
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

#[derive(Copy, Clone, Eq, PartialEq, FromArg, Default)]
enum EnableColor {
    #[default]
    Auto,
    Always,
    Never,
}

impl Args {
    fn top_level(&mut self, args: &mut Pareg) -> Result<()> {
        fn opt_iter(arg: &str) -> Pareg {
            if arg.is_empty() {
                vec![].into()
            } else {
                vec![arg.to_string()].into()
            }
        }

        while let Some(a) = args.next() {
            match a {
                "i" | "instance" => self.instance(args)?,
                "h" | "help" => help(args, self),
                "run" => self.run(args)?,
                "-h" | "--help" | "-?" => {
                    self.should_exit = true;
                    help_all(self.stdout_color);
                }
                "--version" => {
                    self.should_exit = true;
                    help_version(self.stdout_color);
                }
                "-p" | "--port" => {
                    self.port = args.next_arg()?;
                }
                "-a" | "--address" => {
                    self.server_address = args.next_arg()?;
                }
                v if has_any_key!(v, '=', "--color", "--colour") => {
                    self.stdout_color =
                        args.cur_val_or_next::<EnableColor>('=')?.into();
                }
                "--" => {}
                a => {
                    if let Some(i) = a.strip_prefix("-I") {
                        if let Err(Error::ArgParse(e)) =
                            self.instance(&mut opt_iter(i))
                        {
                            args.map_err(Err(e))?;
                        }
                        continue;
                    }

                    if let Some(i) = a.strip_prefix("-R") {
                        self.run(&mut opt_iter(i))?;
                        continue;
                    }

                    if let Some(i) = a.strip_prefix("-H") {
                        help(&mut opt_iter(i), self);
                        continue;
                    }

                    return args.err_unknown_argument().err()?;
                }
            }
        }

        Ok(())
    }

    fn instance(&mut self, args: &mut Pareg) -> Result<()> {
        self.should_exit = true;

        let mut instance = Instance::default();
        instance.parse(args, self.stdout_color)?;

        if !instance.messages.is_empty() {
            self.actions.push(Action::Instance(instance));
        }

        Ok(())
    }

    fn run(&mut self, args: &mut Pareg) -> Result<()> {
        let mut info = Run::default();
        info.parse(args, self.stdout_color)?;

        if info.detach {
            self.should_exit = true;
            self.actions.push(Action::RunDetached(info));
        } else {
            self.run = Some(info)
        }

        Ok(())
    }
}

impl From<EnableColor> for bool {
    fn from(value: EnableColor) -> Self {
        match value {
            EnableColor::Auto => io::stdout().is_terminal(),
            EnableColor::Always => true,
            EnableColor::Never => false,
        }
    }
}
