use std::{net::TcpListener, path::PathBuf, time::Duration};

use log::error;

use crate::{
    app::UampApp,
    core::msg::ComMsg,
    hotkeys::{Action, Hotkey},
};

#[allow(dead_code)] // Some variants are never constructed
#[derive(Debug, Clone)]
pub enum Message {
    AddSearchPath(PathBuf),
    RemoveSearchPath(usize),
    AddAudioExtension(String),
    RemoveAudioExtension(usize),
    AddGlobalHotkey(String, String),
    RemoveGlobalHotkey(usize),
    ServerAddress(String),
    RecursiveSearch(bool),
    UpdateLibraryOnStart(bool),
    RegisterGlobalHotkeys(bool),
    VolumeJump(f32),
    SaveTimeout(Option<Duration>),
    FadePlayPause(Duration),
    Gapless(bool),
    TickLength(Duration),
    SeekJump(Duration),
    Port(u16),
    DeleteLogsAfter(Duration),
    EnableServer(bool),
}

impl UampApp {
    pub fn config_event(&mut self, msg: Message) -> ComMsg {
        match msg {
            Message::AddSearchPath(p) => {
                if !self.config.search_paths().contains(&p) {
                    self.config.search_paths_mut().push(p);
                }
            }
            Message::RemoveSearchPath(i) => {
                self.config.search_paths_mut().remove(i);
            }
            Message::AddAudioExtension(s) => {
                if !self.config.audio_extensions().contains(&s) {
                    self.config.audio_extensions_mut().push(s);
                }
            }
            Message::RemoveAudioExtension(i) => {
                self.config.audio_extensions_mut().remove(i);
            }
            Message::AddGlobalHotkey(h, a) => {
                let hotkey = match h.parse::<Hotkey>() {
                    Ok(h) => h,
                    Err(e) => {
                        error!("Failed to parse hotkey: {e}");
                        return ComMsg::none();
                    }
                };
                let action = match a.parse::<Action>() {
                    Ok(a) => a,
                    Err(e) => {
                        error!("Failed to parse hotkey action: {e}");
                        return ComMsg::none();
                    }
                };
                self.config.global_hotkeys_mut().insert(h, a);
                if let Err(e) = self.hotkey_mgr.add_hotkey(hotkey, action) {
                    error!("Failed to register hotkey: {e}");
                }
            }
            Message::RemoveGlobalHotkey(i) => {
                let (h, a) = if let Some((h, a)) =
                    self.config.global_hotkeys().iter().nth(i)
                {
                    (h.clone(), a.clone())
                } else {
                    return ComMsg::none();
                };
                self.config.global_hotkeys_mut().remove(&h);

                let hotkey = match h.parse::<Hotkey>() {
                    Ok(h) => h,
                    Err(e) => {
                        error!("Failed to parse hotkey: {e}");
                        return ComMsg::none();
                    }
                };
                let action = match a.parse::<Action>() {
                    Ok(a) => a,
                    Err(e) => {
                        error!("Failed to parse hotkey action: {e}");
                        return ComMsg::none();
                    }
                };
                if let Err(e) = self.hotkey_mgr.remove_hotkey(&hotkey, &action)
                {
                    error!("Failed to unregister hotkey: {e}");
                }
            }
            Message::ServerAddress(s) => {
                if self.config.server_address() != &s {
                    match TcpListener::bind(format!(
                        "{}:{}",
                        s,
                        self.config.port()
                    )) {
                        Ok(l) => {
                            self.stop_server();
                            self.listener.set(Some(l));
                            *self.config.server_address_mut() = s;
                        }
                        Err(e) => error!("Failed to create server: {e}"),
                    }
                }
            }
            Message::RecursiveSearch(b) => {
                self.config.recursive_search_set(b);
            }
            Message::UpdateLibraryOnStart(b) => {
                self.config.update_library_on_start_set(b);
            }
            Message::RegisterGlobalHotkeys(b) => {
                if !self.config.register_global_hotkeys_set(b) {
                    return ComMsg::none();
                }
                if b {
                    self.register_global_hotkeys();
                } else {
                    self.hotkey_mgr.disable();
                }
            }
            Message::VolumeJump(f) => {
                self.config.volume_jump_set(f);
            }
            Message::SaveTimeout(od) => {
                self.config.save_timeout_set(od.map(|d| d.into()));
            }
            Message::FadePlayPause(d) => {
                self.config.fade_play_pause_set(d.into());
                todo!("Update fade in player")
            }
            Message::Gapless(b) => {
                self.config.gapless_set(b);
                todo!("Update gapless in player")
            }
            Message::TickLength(d) => {
                self.config.tick_length_set(d.into());
            }
            Message::SeekJump(d) => {
                self.config.seek_jump_set(d.into());
            }
            Message::Port(u) => {
                if self.config.port() != u {
                    match TcpListener::bind(format!(
                        "{}:{}",
                        self.config.server_address(),
                        u
                    )) {
                        Ok(l) => {
                            self.stop_server();
                            self.listener.set(Some(l));
                            self.config.port_set(u);
                        }
                        Err(e) => error!("Failed to create server: {e}"),
                    }
                }
            }
            Message::DeleteLogsAfter(d) => {
                self.config.delete_logs_after_set(d.into());
            }
            Message::EnableServer(b) => {
                if self.config.enable_server() != b {
                    if b {
                        match TcpListener::bind(format!(
                            "{}:{}",
                            self.config.server_address(),
                            self.config.port()
                        )) {
                            Ok(l) => {
                                self.listener.set(Some(l));
                                self.config.enable_server_set(b);
                            }
                            Err(e) => error!("Failed to create server: {e}"),
                        }
                    } else {
                        self.stop_server();
                        self.config.enable_server_set(b);
                    }
                }
            }
        }

        ComMsg::none()
    }
}
