use std::mem::replace;

use itertools::Itertools;
use log::error;

use crate::{
    app::UampApp,
    config::ConfMessage,
    core::{
        extensions::str_to_duration,
        msg::{ComMsg, Msg},
    },
};

use super::help::SetHelp;

#[derive(Default)]
pub struct SetState {
    pub(super) help: Option<&'static SetHelp>,
    pub(super) category: Category,
    pub(super) extension_state: String,
    pub(super) search_path_state: String,
    pub(super) volume_jump_state: String,
    pub(super) save_timeout_state: String,
    pub(super) seek_jump_state: String,
    pub(super) delete_logs_after_state: String,
    pub(super) hotkey_state: String,
    pub(super) tick_length_state: String,
    pub(super) port_state: String,
    pub(super) server_address_state: String,
    pub(super) fade_play_pause_state: String,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum Category {
    #[default]
    Library,
    Playback,
    Hotkeys,
    Server,
    Other,
}

#[derive(Clone, Debug)]
pub enum SetMessage {
    SetCategory(Category),
    ShowHelp(&'static SetHelp),
    ExtensionInput(String),
    ExtensionConfirm,
    SearchPathInput(String),
    SearchPathConfirm,
    VolumeJumpInput(String),
    VolumeJumpConfirm,
    SaveTimeoutInput(String),
    SaveTimeoutConfirm,
    SeekJumpInput(String),
    SeekJumpConfirm,
    DeleteLogsAfterInput(String),
    DeleteLogsAfterConfirm,
    HotkeyInput(String),
    HotkeyConfirm,
    TickLengthInput(String),
    TickLengthConfirm,
    PortInput(String),
    PortConfirm,
    ServerAddressInput(String),
    ServerAddressConfirm,
    FadePlayPauseInput(String),
    FadePlayPauseConfirm,
}

impl UampApp {
    pub(super) fn settings_event_inner(&mut self, msg: SetMessage) -> ComMsg {
        match msg {
            SetMessage::SetCategory(c) => {
                self.gui.set_state.category = c;
                self.gui.set_state.help = None;
            }
            SetMessage::ShowHelp(h) => {
                if let Some(oh) = self.gui.set_state.help {
                    if !std::ptr::eq(oh, h) {
                        //self.gui.wb_states[WB_SETTINGS_HELP].get_mut()
                    }
                }
                self.gui.set_state.help = Some(h);
            }
            SetMessage::ExtensionInput(s) => {
                self.gui.set_state.extension_state = s
            }
            SetMessage::ExtensionConfirm => {
                let s = replace(
                    &mut self.gui.set_state.extension_state,
                    String::new(),
                );
                return ComMsg::Msg(Msg::Config(
                    ConfMessage::AddAudioExtension(s),
                ));
            }
            SetMessage::SearchPathInput(s) => {
                self.gui.set_state.search_path_state = s
            }
            SetMessage::SearchPathConfirm => {
                let s = replace(
                    &mut self.gui.set_state.search_path_state,
                    String::new(),
                );
                return ComMsg::Msg(Msg::Config(ConfMessage::AddSearchPath(
                    s.into(),
                )));
            }
            SetMessage::VolumeJumpInput(s) => {
                self.gui.set_state.volume_jump_state = s
            }
            SetMessage::VolumeJumpConfirm => {
                let s = replace(
                    &mut self.gui.set_state.volume_jump_state,
                    String::new(),
                );
                match s.parse::<f32>() {
                    Ok(f) => {
                        return ComMsg::Msg(Msg::Config(
                            ConfMessage::VolumeJump(f / 100.),
                        ))
                    }
                    Err(e) => {
                        error!("Failed to parse volume jump: {e}");
                    }
                }
            }
            SetMessage::SaveTimeoutInput(s) => {
                self.gui.set_state.save_timeout_state = s
            }
            SetMessage::SaveTimeoutConfirm => {
                let s = replace(
                    &mut self.gui.set_state.save_timeout_state,
                    String::new(),
                );
                if s.is_empty() {
                    return ComMsg::Msg(Msg::Config(
                        ConfMessage::SaveTimeout(None),
                    ));
                }
                match str_to_duration(&s) {
                    Some(d) => {
                        return ComMsg::Msg(Msg::Config(
                            ConfMessage::SaveTimeout(Some(d)),
                        ))
                    }
                    None => {
                        error!("Failed to parse save timeout");
                    }
                }
            }
            SetMessage::SeekJumpInput(s) => {
                self.gui.set_state.seek_jump_state = s
            }
            SetMessage::SeekJumpConfirm => {
                let s = replace(
                    &mut self.gui.set_state.seek_jump_state,
                    String::new(),
                );
                match str_to_duration(&s) {
                    Some(d) => {
                        return ComMsg::Msg(Msg::Config(
                            ConfMessage::SeekJump(d),
                        ))
                    }
                    None => {
                        error!("Failed to parse seek jump");
                    }
                }
            }
            SetMessage::DeleteLogsAfterInput(s) => {
                self.gui.set_state.delete_logs_after_state = s
            }
            SetMessage::DeleteLogsAfterConfirm => {
                let s = replace(
                    &mut self.gui.set_state.delete_logs_after_state,
                    String::new(),
                );
                match str_to_duration(&s) {
                    Some(d) => {
                        return ComMsg::Msg(Msg::Config(
                            ConfMessage::DeleteLogsAfter(d),
                        ))
                    }
                    None => {
                        error!("Failed to parse log timeout");
                    }
                }
            }
            SetMessage::HotkeyInput(s) => self.gui.set_state.hotkey_state = s,
            SetMessage::HotkeyConfirm => {
                let s = replace(
                    &mut self.gui.set_state.hotkey_state,
                    String::new(),
                );
                let s = s.split(':').map(|s| s.trim()).collect_vec();
                return ComMsg::Msg(Msg::Config(
                    ConfMessage::AddGlobalHotkey(
                        s[0].to_string(),
                        s[1].to_string(),
                    ),
                ));
            }
            SetMessage::TickLengthInput(s) => {
                self.gui.set_state.tick_length_state = s
            }
            SetMessage::TickLengthConfirm => {
                let s = replace(
                    &mut self.gui.set_state.tick_length_state,
                    String::new(),
                );
                match str_to_duration(&s) {
                    Some(d) => {
                        return ComMsg::Msg(Msg::Config(
                            ConfMessage::TickLength(d),
                        ))
                    }
                    None => {
                        error!("Failed to parse tick length");
                    }
                }
            }
            SetMessage::PortInput(s) => self.gui.set_state.port_state = s,
            SetMessage::PortConfirm => {
                let s =
                    replace(&mut self.gui.set_state.port_state, String::new());
                match s.parse::<u16>() {
                    Ok(u) => {
                        return ComMsg::Msg(Msg::Config(ConfMessage::Port(u)))
                    }
                    Err(e) => {
                        error!("Failed to parse server port: {e}");
                    }
                }
            }
            SetMessage::ServerAddressInput(s) => {
                self.gui.set_state.server_address_state = s
            }
            SetMessage::ServerAddressConfirm => {
                let s = replace(
                    &mut self.gui.set_state.server_address_state,
                    String::new(),
                );
                return ComMsg::Msg(Msg::Config(ConfMessage::ServerAddress(
                    s,
                )));
            }
            SetMessage::FadePlayPauseInput(s) => {
                self.gui.set_state.fade_play_pause_state = s
            }
            SetMessage::FadePlayPauseConfirm => {
                let s = replace(
                    &mut self.gui.set_state.fade_play_pause_state,
                    String::new(),
                );
                match str_to_duration(&s) {
                    Some(d) => {
                        return ComMsg::Msg(Msg::Config(
                            ConfMessage::FadePlayPause(d),
                        ))
                    }
                    None => {
                        error!("Failed to parse save timeout");
                    }
                }
            }
        }

        ComMsg::none()
    }
}
