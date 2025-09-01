use std::{env, time::Instant};

use bitflags::bitflags;
use notify::Watcher;
use serde_json::from_value;

use crate::core::{
    Alias, AppCtrl, Error, Job, Msg, Result, UampApp,
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

bitflags! {
    struct Change: u64 {
        const LIBRARY_PATH = 0x1;
        const PLAYER_PATH = 0x2;
        const SEARCH_PATHS = 0x4;
        const AUDIO_EXTENSIONS = 0x8;
        const RECURSIVE_SEARCH = 0x10;
        const SERVER_ADDRESS = 0x20;
        const PORT = 0x40;
        const SKIN = 0x80;
        const ENABLE_SERVER = 0x100;
        const AUTO_RESTART = 0x200;
        const SYSTEM_PLAYER = 0x400;
        const CACHE_PATH = 0x800;
    }
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

        new.set_change(true);
        *self = new;

        res
    }

    fn update(&mut self, up: serde_json::Value) -> Result<Change> {
        let serde_json::Value::Object(obj) = up else {
            return Err(Error::invalid_value("Expected json object."));
        };

        let mut change = Change::empty();

        for (k, v) in obj {
            match k.as_str() {
                "library_path" => {
                    let path = from_value(v)?;
                    if *self.library_path() != path {
                        *self.library_path_mut() = path;
                        change |= Change::LIBRARY_PATH;
                    }
                }
                "player_path" => {
                    let path = from_value(v)?;
                    if *self.player_path() != path {
                        *self.player_path_mut() = path;
                        change |= Change::PLAYER_PATH;
                    }
                }
                "cache_path" => {
                    let path = from_value(v)?;
                    if path != *self.cache_path() {
                        *self.cache_path_mut() = path;
                        change |= Change::CACHE_PATH;
                    }
                }
                "search_paths" => {
                    let paths = from_value(v)?;
                    if paths != *self.search_paths() {
                        *self.search_paths_mut() = paths;
                        change |= Change::SEARCH_PATHS;
                    }
                }
                "audio_extensions" => {
                    let exts = from_value(v)?;
                    if exts != *self.audio_extensions() {
                        *self.audio_extensions_mut() = exts;
                        change |= Change::AUDIO_EXTENSIONS;
                    }
                }
                "recursive_search" => {
                    let rc = from_value(v)?;
                    if rc != self.recursive_search() {
                        self.recursive_search_set(rc);
                        change |= Change::RECURSIVE_SEARCH;
                    }
                }
                "server_address" => {
                    let adr = from_value(v)?;
                    if adr != *self.server_address() {
                        *self.server_address_mut() = adr;
                        change |= Change::SERVER_ADDRESS;
                    }
                }
                "port" => {
                    let port = from_value(v)?;
                    if port != self.port() {
                        self.port_set(port);
                        change |= Change::PORT;
                    }
                }
                "skin" => {
                    let skin = from_value(v)?;
                    if skin != *self.skin() {
                        *self.skin_mut() = skin;
                        change |= Change::SKIN;
                    }
                }
                "update_mode" => *self.update_mode_mut() = from_value(v)?,
                "update_remote" => *self.update_remote_mut() = from_value(v)?,
                "delete_logs_after" => {
                    _ = self.delete_logs_after_set(from_value(v)?)
                }
                "enable_server" => {
                    let enable = from_value(v)?;
                    if enable != self.enable_server() {
                        self.enable_server_set(enable);
                        change |= Change::ENABLE_SERVER;
                    }
                }
                "auto_restart" => {
                    let ar = from_value(v)?;
                    if ar != self.auto_restart() {
                        self.auto_restart_set(ar);
                        change |= Change::AUTO_RESTART;
                    }
                }
                "control_aliases" => {
                    *self.control_aliases_mut() = from_value(v)?
                }
                "default_playlist_end_action" => {
                    *self.default_playlist_end_action_mut() = from_value(v)?
                }
                "simple_sorting" => {
                    _ = self.simple_sorting_set(from_value(v)?)
                }
                "play_on_start" => _ = self.play_on_start_set(from_value(v)?),
                "shuffle_current" => {
                    _ = self.shuffle_current_set(from_value(v)?)
                }
                "update_library_on_start" => {
                    _ = self.update_library_on_start_set(from_value(v)?)
                }
                "remove_missin_on_load" => {
                    _ = self.remove_missing_on_load_set(from_value(v)?)
                }
                "volume_jump" => _ = self.volume_jump_set(from_value(v)?),
                "save_playback_pos" => {
                    _ = self.save_playback_pos_set(from_value(v)?)
                }
                "save_timeout" => _ = self.save_timeout_set(from_value(v)?),
                "fade_play_pause" => {
                    _ = self.fade_play_pause_set(from_value(v)?)
                }
                "gapless" => _ = self.gapless_set(from_value(v)?),
                "seek_jump" => _ = self.seek_jump_set(from_value(v)?),
                "client_image_lookup" => {
                    _ = self.client_image_lookup_set(from_value(v)?)
                }
                "system_player" => {
                    let sp = from_value(v)?;
                    if sp != self.system_player() {
                        self.system_player_set(sp);
                        change |= Change::SYSTEM_PLAYER;
                    }
                }
                "web_client_command" => {
                    *self.web_client_command_mut() = from_value(v)?;
                }
                "default_run_type" => {
                    self.default_run_type_set(from_value(v)?);
                }
                k => {
                    return Err(Error::invalid_value(format!(
                        "Invalid setting key `{k}`."
                    )));
                }
            }
        }

        Ok(change)
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
                self.config.set_change(false);
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
