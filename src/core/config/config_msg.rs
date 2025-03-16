use std::{net::TcpStream, time::Instant};

use crate::{
    core::{
        Alias, Error, Msg, Result, TaskType, UampApp,
        messenger::{Messenger, MsgMessage},
    },
    env::AppCtrl,
};

use super::Config;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Updates configuration.
#[derive(Debug, Clone)]
pub enum ConfigMsg {
    /// Reload the configuration from file.
    Reload,
}

impl UampApp {
    /// Handles event for the configuration.
    ///
    /// # Returns
    /// May return newly generated message that should be handled.
    pub(in crate::core) fn config_event(
        &mut self,
        ctrl: &mut AppCtrl,
        msg: ConfigMsg,
    ) -> Result<Vec<Msg>> {
        match msg {
            ConfigMsg::Reload => {
                let path =
                    self.config.config_path.as_ref().ok_or_else(|| {
                        Error::invalid_operation()
                            .msg("Cannot reload config.")
                            .reason("Config save is disabled.")
                            .warn()
                    })?;

                let mut conf = Config::from_json(path)
                    .map_err(|e| e.prepend("Failed to reload config."))?;

                let reload_server = (conf.server_address()
                    != self.config.server_address()
                    || conf.port() != self.config.port()
                    || conf.enable_server() != self.config.enable_server())
                .then(|| {
                    (self.config.server_address().clone(), self.config.port())
                });
                self.player.load_config(&conf);
                conf.force_server = self.config.force_server;
                self.config = conf;
                if let Some((adr, port)) = reload_server {
                    self.reload_server(ctrl, adr, port)
                        .map_err(|e| e.prepend("Failed to reload server."))?;
                }
            }
        }

        Ok(vec![])
    }

    pub(in crate::core) fn config_routine(
        &mut self,
        ctrl: &mut AppCtrl,
        now: Instant,
    ) -> Result<()> {
        if self
            .config
            .save_timeout()
            .map(|t| now - self.last_save >= t.0)
            .unwrap_or_default()
        {
            self.save_all(false, ctrl)
        } else {
            Ok(())
        }
    }

    pub fn invoke_alias(&mut self, alias: &Alias) -> Result<Vec<Msg>> {
        let al = self.config.control_aliases().get(&alias.name).ok_or_else(
            || {
                Error::invalid_operation()
                    .msg("Cannot invoke alias.")
                    .reason(format!("Unknown alias name `{}`.", alias.name))
            },
        )?;
        al.get_msg_vec(&alias.args)
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl UampApp {
    fn reload_server(
        &mut self,
        ctrl: &mut AppCtrl,
        old_adr: String,
        old_port: u16,
    ) -> Result<()> {
        if ctrl.any_task(|t| t == TaskType::Server) {
            let stream = TcpStream::connect(format!("{old_adr}:{old_port}"))
                .map_err(|e| {
                Error::io(e)
                    .msg("Failed to reload server.")
                    .reason("Couldn't connect to the server.")
            })?;
            let mut msgr = Messenger::new(&stream);
            msgr.send(MsgMessage::Stop)?;
        } else if self.config.enable_server() || self.config.force_server {
            Self::start_server(&self.config, ctrl, self.sender.clone())?;
        }

        Ok(())
    }
}
