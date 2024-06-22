use std::{net::TcpStream, time::Instant};

use log::{error, warn};

use crate::{
    core::{
        messenger::{Messenger, MsgMessage},
        Msg, Result, TaskType, UampApp,
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
    ) -> Option<Msg> {
        match msg {
            ConfigMsg::Reload => {
                let Some(path) = self.config.config_path.as_ref() else {
                    warn!("Cannot reaload config because the path is unknwn");
                    return None;
                };

                let mut conf = match Config::from_json(path) {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("Failed to reload config: {e}");
                        return None;
                    }
                };

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
                    if let Err(e) = self.reload_server(ctrl, adr, port) {
                        error!("Failed to reload server: {e}");
                    }
                }
            }
        }

        None
    }

    pub(in crate::core) fn config_update(
        &mut self,
        ctrl: &mut AppCtrl,
        now: Instant,
    ) {
        if self
            .config
            .save_timeout()
            .map(|t| now - self.last_save >= t.0)
            .unwrap_or_default()
        {
            self.save_all(false, ctrl);
        }
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
            let stream = TcpStream::connect(format!("{old_adr}:{old_port}"))?;
            let mut msgr = Messenger::new(&stream);
            msgr.send(MsgMessage::Stop)?;
        } else if self.config.enable_server() || self.config.force_server {
            Self::start_server(&self.config, ctrl, self.sender.clone())?;
        }

        Ok(())
    }
}
