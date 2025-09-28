use std::{env, time::Instant};

use notify::Watcher;

use crate::core::{
    Alias, AppCtrl, Error, Job, Msg, Result, UampApp,
    config::Change,
    library::LoadOpts,
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
    Set(serde_json::Value),
}

impl Config {
    fn set_new(&mut self, new: Config) -> Change {
        let mut res = Change::empty();

        if self.library_path() != new.library_path() {
            res |= Change::LIBRARY_PATH;
        }
        if self.player_path() != new.player_path() {
            res |= Change::PLAYER_PATH;
        }
        if self.search_paths() != new.search_paths() {
            res |= Change::SEARCH_PATHS;
        }
        if self.audio_extensions() != new.audio_extensions() {
            res |= Change::AUDIO_EXTENSIONS;
        }
        if self.server_address() != new.server_address() {
            res |= Change::SERVER_ADDRESS;
        }
        if self.port() != new.port() {
            res |= Change::PORT;
        }
        if self.skin() != new.skin() {
            res |= Change::SKIN;
        }
        if self.enable_server() != new.enable_server() {
            res |= Change::ENABLE_SERVER;
        }
        if self.auto_restart() != new.auto_restart() {
            res |= Change::AUTO_RESTART;
        }
        if self.system_player() != new.system_player() {
            res |= Change::SYSTEM_PLAYER;
        }
        if self.cache_path() != new.cache_path() {
            res |= Change::CACHE_PATH;
        }
        if self.fade_play_pause() != new.fade_play_pause() {
            res |= Change::FADE_PLAY_PAUSE;
        }
        if self.gapless() != new.gapless() {
            res |= Change::GAPLESS;
        }

        new.change();
        *self = new;

        res
    }
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

                let change = self.config.set_new(conf);
                self.propagate_config_change(ctrl, change)?;
                self.config.reset_change();
            }
            ConfigMsg::Set(cfg) => {
                let change = self.config.update(cfg)?;
                self.propagate_config_change(ctrl, change)?;
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
    fn propagate_config_change(
        &mut self,
        ctrl: &mut AppCtrl,
        change: Change,
    ) -> Result<()> {
        if change.contains(Change::LIBRARY_PATH) {
            self.library.change();
        }

        if change.contains(Change::PLAYER_PATH) {
            self.player.change();
        }

        if change.intersects(
            Change::SEARCH_PATHS
                | Change::AUDIO_EXTENSIONS
                | Change::RECURSIVE_SEARCH,
        ) && self.config.update_library_on_start()
        {
            self.start_get_new_songs(ctrl, LoadOpts::default())?;
        }

        let restart_server = change.intersects(
            Change::SERVER_ADDRESS | Change::PORT | Change::ENABLE_SERVER,
        );
        if restart_server {
            self.reload_server(ctrl)?;
        }

        if !restart_server && let Some(ref d) = self.jobs.server {
            if change.contains(Change::CACHE_PATH) {
                *d.cache.write().unwrap() = self.config.cache_path().clone();
            }
            if change.contains(Change::SKIN) {
                *d.client.write().unwrap() = self.config.skin().clone();
                self.client_update(SubMsg::ClientChanged);
            }
        }

        if change.contains(Change::AUTO_RESTART) {
            if self.config.auto_restart() {
                self.enable_auto_restart()?;
            } else {
                self.disable_auto_restart()?;
            }
        }

        if change.contains(Change::SYSTEM_PLAYER) {
            if self.config.system_player() {
                self.enable_system_player();
            } else {
                self.disable_system_player();
            }
        }

        if change.contains(Change::FADE_PLAY_PAUSE) {
            self.player.fade_play_pause(self.config.fade_play_pause().0);
        }

        if change.contains(Change::GAPLESS) {
            self.player.gapless(self.config.gapless());
        }

        if self.config.changed() {
            self.client_update(SubMsg::ConfigChanged(
                sub::Config::new(&self.config).into(),
            ));
        }

        Ok(())
    }

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

    fn enable_auto_restart(&mut self) -> Result<()> {
        let Some(ref mut fw) = self.file_watch else {
            self.file_watch =
                Self::watch_files(self.rt.andle(), &self.config)?;
            return Ok(());
        };

        let exe = env::current_exe()?;
        fw.watch(&exe, notify::RecursiveMode::NonRecursive)?;

        Ok(())
    }

    fn disable_auto_restart(&mut self) -> Result<()> {
        let Some(ref mut fw) = self.file_watch else {
            return Ok(());
        };

        let exe = env::current_exe()?;
        fw.unwatch(&exe)?;

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
