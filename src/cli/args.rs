use crate::{
    config::Config,
    core::messenger::{msg::Request, MsgMessage},
    next,
};

use super::{
    err::{Error, Result},
    help::help,
    parsers::parse_control_message,
    Action,
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
    pub must_run: bool,
}

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

impl Args {
    /// Parses the CLI arguments and returns the parsed arguments
    pub fn parse<'a>(args: impl Iterator<Item = &'a str>) -> Result<Self> {
        let mut res = Args::default();

        let mut args = args.skip(1);

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
    /// Parses the instance action arguments
    fn instance<'a>(
        &mut self,
        args: &mut impl Iterator<Item = &'a str>,
    ) -> Result<()> {
        self.should_exit = true;
        let a = next!(args);

        match a {
            "info" => self
                .actions
                .push(Action::Message(MsgMessage::Request(Request::Info))),
            "--" => {
                return Err(Error::UnexpectedEnd(Some("instance".to_owned())))
            }
            _ => {
                let msg = parse_control_message(a)?;
                self.actions.push(Action::control(msg));
            }
        }

        Ok(())
    }

    fn top_level<'a>(
        &mut self,
        args: &mut impl Iterator<Item = &'a str>,
    ) -> Result<()> {
        while let Some(a) = args.next() {
            match a {
                "i" | "instance" => self.instance(args)?,
                "h" | "help" | "-h" | "--help" | "-?" => help(args, self)?,
                "-p" | "--port" => {
                    self.port = Some(next!(u16, args, Some(a.to_owned())))
                }
                "-a" | "--address" => {
                    self.server_address = Some(next!(args).to_owned())
                }
                a => return Err(Error::UnknownArgument(Some(a.to_owned()))),
            }
        }

        Ok(())
    }
}
