use std::time::Instant;

use crate::core::{
    Alias, AppCtrl, Error, Job, Msg, Result, UampApp,
    server::{SubMsg, sub::NewServer},
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

                let reload_server = conf.server_address()
                    != self.config.server_address()
                    || conf.port() != self.config.port()
                    || conf.enable_server() != self.config.enable_server();

                let system_player_change =
                    conf.system_player() != self.config.system_player();

                if !reload_server && let Some(ref d) = self.jobs.server {
                    if conf.cache_path() != self.config.cache_path() {
                        *d.cache.write().unwrap() = conf.cache_path().clone();
                    }
                    if conf.http_client() != self.config.http_client() {
                        *d.client.write().unwrap() = conf.http_client().clone();
                        self.client_update(SubMsg::ClientChanged);
                    }
                }

                self.player.load_config(&conf);
                conf.force_server = self.config.force_server;
                self.config = conf;

                if system_player_change {
                    if self.config.system_player() {
                        self.enable_system_player();
                    } else {
                        self.disable_system_player();
                    }
                }

                if reload_server {
                    self.reload_server(ctrl)
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
    fn reload_server(&mut self, ctrl: &mut AppCtrl) -> Result<()> {
        if self.jobs.is_running(Job::SERVER) {
            self.client_update(SubMsg::NewServer(NewServer::new(
                self.config.port(),
                self.config.server_address().clone().into(),
            )));
            self.stop_server();
        } else if self.config.should_start_server() {
            self.start_server(ctrl)?;
        }

        Ok(())
    }

    pub fn shutdown_server(&mut self) {
        self.stop_server();
        self.config.force_server = Some(false);
    }

    pub fn stop_server(&mut self) {
        if self.jobs.is_running(Job::SERVER)
            && let Some(d) = self.jobs.server.take()
        {
            d.cancel.cancel();
        }
    }
}
