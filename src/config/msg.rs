use std::{path::PathBuf, time::Duration};

use crate::{app::UampApp, core::msg::ComMsg};

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
                self.config.search_paths_mut().push(p);
            }
            Message::RemoveSearchPath(i) => {
                self.config.search_paths_mut().remove(i);
            }
            Message::AddAudioExtension(s) => {
                self.config.audio_extensions_mut().push(s);
            }
            Message::RemoveAudioExtension(i) => {
                self.config.audio_extensions_mut().remove(i);
            }
            Message::AddGlobalHotkey(h, a) => {
                self.config.global_hotkeys_mut().add_hotkey(h, a);
                todo!("refresh the hotkeys")
            }
            Message::RemoveGlobalHotkey(i) => {
                todo!("remove hotkey at index {i}, refresh hotkeys")
            }
            Message::ServerAddress(s) => {
                *self.config.server_address_mut() = s;
                todo!("restart server")
            }
            Message::RecursiveSearch(b) => {
                self.config.recursive_search_set(b);
            }
            Message::UpdateLibraryOnStart(b) => {
                self.config.update_library_on_start_set(b);
            }
            Message::RegisterGlobalHotkeys(b) => {
                self.config.register_global_hotkeys_set(b);
                todo!("unregister/register global hotkeys")
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
                todo!("Refresh tick");
            }
            Message::SeekJump(d) => {
                self.config.seek_jump_set(d.into());
            }
            Message::Port(u) => {
                self.config.port_set(u);
                todo!("Restart server")
            }
            Message::DeleteLogsAfter(d) => {
                self.config.delete_logs_after_set(d.into())
            }
            Message::EnableServer(b) => {
                self.config.enable_server_set(b);
                todo!("start/stop server")
            }
        }

        ComMsg::none()
    }
}