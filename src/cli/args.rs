use pareg::{ArgError, ArgIterator, ByRef};

use crate::{
    config::Config,
    core::{
        err::Result,
        messenger::{msg::Request, MsgMessage},
    },
};

use super::{
    help::{help, help_instance, print_help},
    Action, RunInfo,
};

/// Contains the CLI arguments values
#[derive(Default)]
pub struct Args {
    /// Actions to do
    pub actions: Vec<Action>,

    pub port: Option<u16>,
    pub server_address: Option<String>,

    /// The gui should not run, unless `must_run` is set to `true`
    pub should_exit: bool,
    /// The gui should run in all cases if this is `true`
    pub run: Option<RunInfo>,
}

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

impl Args {
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
        while let Some(a) = args.next() {
            match a {
                "i" | "instance" => self.instance(args)?,
                "h" | "help" => help(args, self)?,
                "run" => self.run(args)?,
                "-h" | "--help" | "-?" => {
                    self.should_exit = true;
                    print_help();
                }
                "-p" | "--port" => {
                    self.port = args.next_arg()?;
                }
                "-a" | "--address" => {
                    self.server_address = args.next_arg()?;
                }
                "--" => {}
                a => Err(ArgError::UnknownArgument(a.into()))?,
            }
        }

        Ok(())
    }

    /// Parses the instance action arguments
    fn instance<'a, I>(&mut self, args: &mut ArgIterator<'a, I>) -> Result<()>
    where
        I: Iterator,
        I::Item: ByRef<&'a str>,
    {
        self.should_exit = true;
        let mut msgs = vec![];

        while let Some(arg) = args.next() {
            match arg {
                "info" => msgs.push(MsgMessage::Request(Request::Info)),
                "-h" | "--help" => help_instance(),
                "--" => break,
                _ => msgs.push(MsgMessage::Control(args.cur_arg()?)),
            }
        }

        if !msgs.is_empty() {
            self.actions.push(Action::Message(msgs));
        }

        Ok(())
    }

    fn run<'a, I>(&mut self, args: &mut ArgIterator<'a, I>) -> Result<()>
    where
        I: Iterator,
        I::Item: ByRef<&'a str>,
    {
        let mut info = RunInfo::default();
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
