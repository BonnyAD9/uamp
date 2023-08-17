use std::{cell::RefCell, net::TcpListener, sync::Arc};

use eyre::Result;
use global_hotkey::{
    hotkey::{self, Code, HotKey},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use iced::{executor, window, Application};
use iced_core::Event;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{
    config::{app_id, default_port, Config},
    library::{Library, SongId},
    messenger::{self, Messenger},
    player::{Player, PlayerMessage},
    theme::Theme,
    uamp_gui::{self, GuiState},
    wid::{Command, Element},
};

pub struct UampApp {
    pub config: Config,
    pub library: Library,
    pub player: Player,

    pub sender: Arc<UnboundedSender<UampMessage>>,
    pub reciever: RefCell<Option<UnboundedReceiver<UampMessage>>>,

    pub theme: Theme,
    pub gui: GuiState,

    pub hotkey_mgr: Option<GlobalHotKeyManager>,
    pub listener: RefCell<Option<TcpListener>>,
}

#[allow(missing_debug_implementations)]
#[derive(Clone, Debug)]
pub enum UampMessage {
    PlaySong(usize, Arc<[SongId]>),
    Control(ControlMsg),
    Gui(uamp_gui::Message),
    Player(PlayerMessage),
}

/// only simple messages that can be safely send across threads and copied
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ControlMsg {
    PlayPause,
    NextSong,
    PrevSong,
    SetVolume(f32),
    VolumeUp,
    VolumeDown,
    ToggleMute,
    Shuffle,
    PlaylistJump(usize),
    Close,
    FindSongs,
}

impl Application for UampApp {
    type Executor = executor::Default;
    type Flags = ();
    type Message = UampMessage;
    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Self, Command) {
        _ = flags;
        (UampApp::default(), Command::none())
    }

    fn update(&mut self, message: Self::Message) -> Command {
        match message {
            UampMessage::PlaySong(index, songs) => {
                self.player.play_playlist(
                    &self.library,
                    songs,
                    Some(index),
                    true,
                );
            }
            UampMessage::Control(msg) => return self.control_event(msg),
            UampMessage::Gui(msg) => return self.gui_event(msg),
            UampMessage::Player(msg) => {
                return self.player.event(&self.library, msg)
            }
        };
        Command::none()
    }

    fn title(&self) -> String {
        app_id()
    }

    fn view(&self) -> Element {
        self.gui()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::Subscription::batch([
            iced::subscription::unfold(
                app_id() + " async msg",
                self.reciever.take(),
                |mut reciever| async {
                    let msg = reciever.as_mut().unwrap().recv().await.unwrap();
                    (msg, reciever)
                },
            ),
            iced::subscription::unfold(
                app_id() + " server",
                self.listener.take(),
                |listener| async {
                    let listener = listener.unwrap();

                    loop {
                        let stream = listener.accept().unwrap();
                        let mut msgr = Messenger::try_new(&stream.0).unwrap();

                        let rec = msgr.recieve();
                        println!("recieve: {rec:?}");

                        let rec = match rec {
                            Ok(m) => m,
                            Err(e) => {
                                _ = msgr.send(messenger::error(
                                    messenger::ErrorType::DeserializeFailed,
                                    e.to_string(),
                                ));
                                continue;
                            }
                        };

                        let (response, msg) = Self::message_event(rec);
                        _ = msgr.send(response);

                        if let Some(msg) = msg {
                            break (msg, Some(listener));
                        } else {
                            continue;
                        }
                    }
                },
            ),
            iced::subscription::events_with(|e, _| match e {
                Event::Window(window::Event::CloseRequested) => {
                    Some(UampMessage::Control(ControlMsg::Close))
                }
                _ => None,
            }),
        ])
    }
}

impl Default for UampApp {
    fn default() -> Self {
        let conf = Config::from_default_json();

        let mut lib = Library::from_config(&conf);
        if conf.update_library_on_start {
            lib.get_new_songs(&conf);
        }

        let (sender, reciever) = mpsc::unbounded_channel::<UampMessage>();
        let sender = Arc::new(sender);

        let player = Player::from_config(sender.clone(), &conf);

        let hotkey_mgr = if conf.register_global_hotkeys {
            Self::register_hotkeys(sender.clone()).ok()
        } else {
            None
        };

        UampApp {
            config: conf,
            library: lib,
            player,

            sender,
            reciever: RefCell::new(Some(reciever)),

            theme: Theme::default(),
            gui: GuiState::default(),

            hotkey_mgr,
            listener: RefCell::new(Self::start_server().ok()),
        }
    }
}

impl UampApp {
    fn control_event(&mut self, msg: ControlMsg) -> Command {
        match msg {
            ControlMsg::PlayPause => {
                self.player.play_pause(&self.library);
            }
            ControlMsg::NextSong => self.player.play_next(&self.library),
            ControlMsg::PrevSong => self.player.play_prev(&self.library),
            ControlMsg::Close => {
                _ = self.library.to_json(&self.config.library_path);
                _ = self.player.to_json(&self.config.player_path);
                _ = self.config.to_default_json();
                return window::close();
            }
            ControlMsg::Shuffle => self.player.shuffle(),
            ControlMsg::SetVolume(v) => {
                self.player.set_volume(v.clamp(0., 1.))
            }
            ControlMsg::VolumeUp => self.player.set_volume(
                (self.player.volume() + self.config.volume_jump).clamp(0., 1.),
            ),
            ControlMsg::VolumeDown => self.player.set_volume(
                (self.player.volume() - self.config.volume_jump).clamp(0., 1.),
            ),
            ControlMsg::PlaylistJump(i) => {
                self.player
                    .play_at(&self.library, i, self.player.is_playing())
            }
            ControlMsg::ToggleMute => self.player.toggle_mute(),
            ControlMsg::FindSongs => {
                self.library.get_new_songs(&self.config);
                _ = self.library.to_json(&self.config.library_path);
            }
        };

        Command::none()
    }

    fn register_hotkeys(
        sender: Arc<UnboundedSender<UampMessage>>,
    ) -> Result<GlobalHotKeyManager> {
        macro_rules! hotkey {
            ($first:ident + $second:ident $(-$rest:ident)*) => {
                hotkey!($second - $first $(-$rest)*)
            };
            ($first:ident + $second:ident $(+$tail:ident)+ $(-$rest:ident)*) => {
                hotkey!($second $(+$tail)+ - $first $(-$rest)*)
            };
            ($key:ident - $first:ident $(-$tail:ident)*) => {{
                let key = HotKey::new(Some(hotkey::Modifiers::$first $(| hotkey::Modifiers::$tail)*), Code::$key);
                let id = key.id();
                (key, id)
            }};
        }

        macro_rules! make_hotkeys {
            ($($key:ident $(+$mods:ident)+ -> $name:ident : $action:ident),+ $(,)?) => {{
                let hotkey_mgr = GlobalHotKeyManager::new()?;

                $(let $name = hotkey!($key $(+$mods)+);)+

                hotkey_mgr.register_all(&[
                    $($name.0),+
                ])?;

                GlobalHotKeyEvent::set_event_handler(Some(
                    move |e: GlobalHotKeyEvent| {
                        match e.id {$(
                            id if id == $name.1 => {
                                _ = sender
                                    .send(UampMessage::Control(ControlMsg::$action));
                            })+
                            _ => {}
                        };
                    },
                ));

                Ok(hotkey_mgr)
            }};
        }

        make_hotkeys!(
            CONTROL + ALT + Home -> play: PlayPause,
            CONTROL + ALT + PageUp -> next: PrevSong,
            CONTROL + ALT + PageDown -> prev: NextSong,
            CONTROL + ALT + ArrowUp -> vol_up: VolumeUp,
            CONTROL + ALT + ArrowDown -> vol_down: VolumeDown,
        )
    }

    fn start_server() -> Result<TcpListener> {
        Ok(TcpListener::bind(format!("127.0.0.1:{}", default_port()))?)
    }
}
