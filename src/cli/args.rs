use crate::{next, config::Config};

use super::{
    help::help, parsers::parse_control_message, Action, err::{Error, Result}
};

/// Contains the CLI arguments values
#[derive(Default)]
pub struct Args {
    /// Actions to do
    pub actions: Vec<Action>,

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

        while let Some(a) = args.next() {
            match a {
                "i" | "instance" => res.instance(&mut args)?,
                "h" | "help" | "-h" | "--help" | "-?" => {
                    help(&mut args, &mut res)?
                }
                a => return Err(Error::UnknownArgument(Some(a.to_owned()))),
            }
        }

        Ok(res)
    }

    pub fn make_config(&self) -> Config {
        Config::from_default_json()
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

        if a == "--" {
            return Err(Error::UnexpectedEnd(Some("instance".to_owned())));
        }

        let msg = parse_control_message(a)?;
        self.actions.push(Action::control(msg));

        Ok(())
    }
}
