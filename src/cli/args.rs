use pareg::{ArgError, ArgIterator, ByRef};

use crate::{cli::help::help_version, config::Config, core::err::Result};

use super::{
    help::{help, help_all},
    Action, Instance, Run,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Contains the CLI arguments values.
#[derive(Default, Debug)]
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
}

impl Args {
    /// Parses the arguments.
    ///
    /// # Returns
    /// The parsed arguments.
    ///
    /// # Errors
    /// - The arguments are invalid.
    pub fn parse<'a, I>(mut args: ArgIterator<'a, I>) -> Result<Self>
    where
        I: Iterator,
        I::Item: ByRef<&'a str>,
    {
        let mut res = Args::default();

        args.next();

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

impl Args {
    fn top_level<'a, I>(&mut self, args: &mut ArgIterator<'a, I>) -> Result<()>
    where
        I: Iterator,
        I::Item: ByRef<&'a str>,
    {
        fn opt_iter(arg: &str) -> impl Iterator<Item = &str> {
            if arg.is_empty() {
                None.into_iter()
            } else {
                Some(arg).into_iter()
            }
        }

        while let Some(a) = args.next() {
            match a {
                "i" | "instance" => self.instance(args)?,
                "h" | "help" => help(args, self),
                "run" => self.run(args)?,
                "-h" | "--help" | "-?" => {
                    self.should_exit = true;
                    help_all();
                }
                "--version" => {
                    self.should_exit = true;
                    help_version();
                }
                "-p" | "--port" => {
                    self.port = args.next_arg()?;
                }
                "-a" | "--address" => {
                    self.server_address = args.next_arg()?;
                }
                "--" => {}
                a => {
                    if let Some(i) = a.strip_prefix("-I") {
                        self.instance(&mut opt_iter(i).into())?;
                        continue;
                    }

                    if let Some(i) = a.strip_prefix("-R") {
                        self.run(&mut opt_iter(i).into())?;
                        continue;
                    }

                    if let Some(i) = a.strip_prefix("-H") {
                        help(&mut opt_iter(i), self);
                        continue;
                    }

                    Err(ArgError::UnknownArgument(a.into()))?
                }
            }
        }

        Ok(())
    }

    fn instance<'a, I>(&mut self, args: &mut ArgIterator<'a, I>) -> Result<()>
    where
        I: Iterator,
        I::Item: ByRef<&'a str>,
    {
        self.should_exit = true;

        let mut instance = Instance::default();
        instance.parse(args)?;

        if !instance.messages.is_empty() {
            self.actions.push(Action::Instance(instance));
        }

        Ok(())
    }

    fn run<'a, I>(&mut self, args: &mut ArgIterator<'a, I>) -> Result<()>
    where
        I: Iterator,
        I::Item: ByRef<&'a str>,
    {
        let mut info = Run::default();
        info.parse(args)?;

        if info.detach {
            self.should_exit = true;
            self.actions.push(Action::RunDetached(info));
        } else {
            self.run = Some(info)
        }

        Ok(())
    }
}
