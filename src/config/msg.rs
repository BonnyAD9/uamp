use std::{net::TcpListener, path::PathBuf, time::Duration};

use log::error;

use crate::{
    app::UampApp,
    core::msg::Msg,
};

use super::config;

#[derive(Debug, Clone)]
pub enum Message {
    Reset(DefMessage),
    AddSearchPath(PathBuf),
    RemoveSearchPath(usize),
    AddAudioExtension(String),
    RemoveAudioExtension(usize),
    ServerAddress(String),
    RecursiveSearch(bool),
    UpdateLibraryOnStart(bool),
    VolumeJump(f32),
    SaveTimeout(Option<Duration>),
    FadePlayPause(Duration),
    Gapless(bool),
    TickLength(Duration),
    SeekJump(Duration),
    Port(u16),
    DeleteLogsAfter(Duration),
    EnableServer(bool),
    ShuffleCurrent(bool),
    ShowHelp(bool),
    PreviousTimeout(Option<Duration>),
    ShowRemainingTime(bool),
    PlayOnStart(bool),
    SimpleSorting(bool),
}

#[derive(Clone, Debug, Copy)]
pub enum DefMessage {
    SearchPaths,
    AudioExtensions,
    ServerAddress,
    RecursiveSearch,
    UpdateLibraryOnStart,
    VolumeJump,
    SaveTimeout,
    FadePlayPause,
    Gapless,
    TickLength,
    SeekJump,
    Port,
    DeleteLogsAfter,
    EnableServer,
    ShuffleCurrent,
    ShowHelp,
    PreviousTimeout,
    ShowRemainingTime,
    PlayOnStart,
    SimpleSorting,
}

impl UampApp {
    pub fn config_event(&mut self, msg: Message) -> Option<Msg> {
        match msg {
            Message::Reset(msg) => {
                return self.reset_event(msg);
            }
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
            Message::ServerAddress(s) => {
                if self.config.server_address() != &s {
                    let adr = format!("{}:{}", s, self.config.port());
                    match TcpListener::bind(&adr) {
                        Ok(l) => {
                            self.stop_server(Some(adr));
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
            Message::VolumeJump(f) => {
                self.config.volume_jump_set(f);
            }
            Message::SaveTimeout(od) => {
                self.config.save_timeout_set(od.map(|d| d.into()));
            }
            Message::FadePlayPause(d) => {
                self.config.fade_play_pause_set(d.into());
                self.player.fade_play_pause(d);
            }
            Message::Gapless(b) => {
                self.config.gapless_set(b);
                self.player.load_config(&self.config);
            }
            Message::TickLength(d) => {
                self.config.tick_length_set(d.into());
            }
            Message::SeekJump(d) => {
                self.config.seek_jump_set(d.into());
            }
            Message::Port(u) => {
                if self.config.port() != u {
                    let adr =
                        format!("{}:{}", self.config.server_address(), u);
                    match TcpListener::bind(&adr) {
                        Ok(l) => {
                            self.stop_server(Some(adr));
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
                        self.stop_server(None);
                        self.config.enable_server_set(b);
                    }
                }
            }
            Message::ShuffleCurrent(b) => {
                if self.config.shuffle_current_set(b) {
                    self.player.shuffle_current = b;
                }
            }
            Message::ShowHelp(b) => {
                self.config.show_help_set(b);
            }
            Message::PreviousTimeout(t) => {
                self.config.previous_timeout_set(t.map(|t| t.into()));
            }
            Message::ShowRemainingTime(b) => {
                self.config.show_remaining_time_set(b);
            }
            Message::PlayOnStart(b) => {
                self.config.play_on_start_set(b);
            }
            Message::SimpleSorting(b) => {
                self.config.simple_sorting_set(b);
            }
        }

        None
    }

    fn reset_event(&mut self, msg: DefMessage) -> Option<Msg> {
        match msg {
            DefMessage::SearchPaths => {
                *self.config.search_paths_mut() =
                    config::default_search_paths();
            }
            DefMessage::AudioExtensions => {
                *self.config.audio_extensions_mut() =
                    config::default_audio_extensions();
            }
            DefMessage::ServerAddress => {
                return Some(Msg::Config(Message::ServerAddress(
                    config::default_server_address(),
                )))
            }
            DefMessage::RecursiveSearch => {
                self.config
                    .recursive_search_set(config::default_recursive_search());
            }
            DefMessage::UpdateLibraryOnStart => {
                self.config.update_library_on_start_set(
                    config::default_update_library_on_start(),
                );
            }
            DefMessage::VolumeJump => {
                self.config.volume_jump_set(config::default_volume_jump());
            }
            DefMessage::SaveTimeout => {
                self.config.save_timeout_set(config::default_save_timeout());
            }
            DefMessage::FadePlayPause => {
                return Some(Msg::Config(Message::FadePlayPause(
                    config::default_fade_play_pause().0,
                )));
            }
            DefMessage::Gapless => {
                return Some(Msg::Config(Message::Gapless(
                    config::default_gapless(),
                )));
            }
            DefMessage::TickLength => {
                return Some(Msg::Config(Message::TickLength(
                    config::default_tick_length().0,
                )));
            }
            DefMessage::SeekJump => {
                self.config.seek_jump_set(config::default_seek_jump());
            }
            DefMessage::Port => {
                return Some(Msg::Config(Message::Port(
                    config::default_port(),
                )));
            }
            DefMessage::DeleteLogsAfter => {
                self.config.delete_logs_after_set(config::default_delete_logs_after());
            }
            DefMessage::EnableServer => {
                return Some(Msg::Config(Message::EnableServer(
                    config::default_enable_server(),
                )));
            }
            DefMessage::ShuffleCurrent => {
                self.config
                    .shuffle_current_set(config::default_shuffle_current());
            }
            DefMessage::ShowHelp => {
                self.config.show_help_set(config::default_show_help());
            }
            DefMessage::PreviousTimeout => {
                self.config
                    .previous_timeout_set(config::default_previous_timeout());
            }
            DefMessage::ShowRemainingTime => {
                self.config.show_remaining_time_set(
                    config::default_show_remaining_time(),
                );
            }
            DefMessage::PlayOnStart => {
                self.config
                    .play_on_start_set(config::default_play_on_start());
            }
            DefMessage::SimpleSorting => {
                self.config
                    .simple_sorting_set(config::default_simple_sorting());
            }
        }

        None
    }
}
