use pareg::{ArgError, ArgIterator, ByRef};

use crate::{config::Config, core::Result};

use super::help::help_run;

#[derive(Default)]
pub struct RunInfo {
    pub detach: bool,

    pub port: Option<u16>,
    pub server_address: Option<String>,
}

impl RunInfo {
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

    pub fn update_config(&self, conf: &mut Config) {
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
