use std::io::{self, IsTerminal};

use pareg::{FromArg, Pareg, has_any_key};

use crate::{
    cli::{help::help_short, port::Port},
    core::{
        Error, Result,
        config::{APP_ID, Config, VERSION_STR},
    },
};

use super::{
    Action, Instance, Props, Run, Shell, help::help, internal::Internal,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Contains the CLI arguments values.
#[derive(Debug, Default)]
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

    /// Shared properties.
    pub props: Props,
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
                "cfg" | "conf" | "config" => self.config(args)?,
                "sh" | "shell" => self.shell(args)?,
                "internal" => self.internal(args)?,
                "-h" | "--help" | "-?" => {
                    self.should_exit = true;
                    help_short(self.props.color);
                }
                "--version" => {
                    self.should_exit = true;
                    println!("{APP_ID} {VERSION_STR}")
                }
                "-p" | "--port" => {
                    self.port = Some(args.next_arg::<Port>()?.0);
                }
                "-a" | "--address" => {
                    self.server_address = Some(args.next_arg()?);
                }
                v if has_any_key!(v, '=', "--color", "--colour") => {
                    self.props.color =
                        args.cur_val_or_next::<EnableColor>('=')?.into();
                }
                "--print" => {
                    self.props.print_style = args.next_arg()?;
                }
                "--" => {}
                a => {
                    if let Some(i) = a.strip_prefix("-I") {
                        if let Err(Error::Pareg(e)) =
                            self.instance(&mut opt_iter(i))
                        {
                            return Err(args.map_err(e).into());
                        }
                        continue;
                    }

                    if let Some(i) = a.strip_prefix("-R") {
                        self.run(&mut opt_iter(i))?;
                        continue;
                    }

                    if let Some(i) = a.strip_prefix("-C") {
                        self.config(&mut opt_iter(i))?;
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
        instance.parse(args, self.props.color)?;

        if !instance.messages.is_empty() {
            self.actions.push(Action::Instance(instance));
        }

        Ok(())
    }

    fn run(&mut self, args: &mut Pareg) -> Result<()> {
        let mut info = Run::default();
        info.parse(args, self.props.color)?;

        if !info.run.unwrap_or(true) {
            self.should_exit = true;
            return Ok(());
        }

        if info.detach {
            self.should_exit = true;
            self.actions.push(Action::RunDetached(info));
        } else {
            self.run = Some(info)
        }

        Ok(())
    }

    fn config(&mut self, args: &mut Pareg) -> Result<()> {
        self.should_exit = true;

        let mut cfg = super::Config::default();
        cfg.parse(args, self.props.color)?;

        if !cfg.actions.is_empty() {
            self.actions.push(Action::Config(cfg));
        }

        Ok(())
    }

    fn shell(&mut self, args: &mut Pareg) -> Result<()> {
        self.should_exit = true;

        let mut sh = Shell::default();
        sh.parse(args, self.props.color)?;
        self.actions.push(Action::Shell(sh));
        Ok(())
    }

    fn internal(&mut self, args: &mut Pareg) -> Result<()> {
        self.should_exit = true;

        let i = Internal::new(args, self.props.color)?;
        if !matches!(i, Internal::None) {
            self.actions.push(Action::Internal(i));
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
