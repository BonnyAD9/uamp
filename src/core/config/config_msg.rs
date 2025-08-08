use std::time::Instant;

use crate::core::{
    Alias, AppCtrl, Error, Job, Msg, Result, UampApp,
    server::{
        SubMsg,
        sub::{self, NewServer},
    },
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
    Set(Box<sub::Config>),
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

                let conf = Config::from_json(path)
                    .map_err(|e| e.prepend("Failed to reload config."))?;

                self.apply_config(conf, ctrl)?;
            }
            ConfigMsg::Set(cfg) => {
                // TODO: optimize
                let mut conf = self.config.clone();
                *conf.library_path_mut() = cfg.library_path;
                *conf.player_path_mut() = cfg.player_path;
                *conf.cache_path_mut() = cfg.cache_path;
                *conf.search_paths_mut() = cfg.search_paths;
                *conf.audio_extensions_mut() = cfg.audio_extensions;
                conf.recursive_search_set(cfg.recursive_search);
                *conf.server_address_mut() = cfg.server_address;
                conf.port_set(cfg.port);
                *conf.skin_mut() = cfg.skin;
                *conf.update_mode_mut() = cfg.update_mode;
                *conf.update_remote_mut() = cfg.update_remote;
                conf.delete_logs_after_set(cfg.delete_logs_after);
                conf.enable_server_set(cfg.enable_server);
                conf.auto_restart_set(cfg.auto_restart);

                *conf.control_aliases_mut() = cfg.control_aliases;
                *conf.default_playlist_end_action_mut() =
                    cfg.default_playlist_end_action;
                conf.simple_sorting_set(cfg.simple_sorting);
                conf.play_on_start_set(cfg.play_on_start);
                conf.shuffle_current_set(cfg.shuffle_current);
                conf.update_library_on_start_set(cfg.update_library_on_start);
                conf.remove_missing_on_load_set(cfg.remove_missing_on_load);
                conf.volume_jump_set(cfg.volume_jump);
                conf.save_playback_pos_set(cfg.save_playback_pos);
                conf.save_timeout_set(cfg.save_timeout);
                conf.fade_play_pause_set(cfg.fade_play_pause);
                conf.gapless_set(cfg.gapless);
                conf.seek_jump_set(cfg.seek_jump);
                conf.client_image_lookup_set(cfg.client_image_lookup);
                conf.system_player_set(cfg.system_player);

                self.apply_config(conf, ctrl)?;

                self.config.to_default_json()?;
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

    fn apply_config(
        &mut self,
        mut conf: Config,
        ctrl: &mut AppCtrl,
    ) -> Result<()> {
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
            if conf.skin() != self.config.skin() {
                *d.client.write().unwrap() = conf.skin().clone();
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

        self.client_update(SubMsg::ConfigChanged(
            sub::Config::new(&self.config).into(),
        ));

        Ok(())
    }
}
