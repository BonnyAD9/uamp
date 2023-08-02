use std::{cell::RefCell, net::TcpListener, sync::Arc};

use eyre::Result;
use global_hotkey::{
    hotkey::{self, Code, HotKey},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use iced::{executor, Application};
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
    PlayPause,
    Gui(uamp_gui::Message),
    Hotkey(HotkeyMessage),
    Player(PlayerMessage),
}

#[derive(Clone, Debug)]
pub enum HotkeyMessage {
    PlayPause,
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
            UampMessage::PlayPause
            | UampMessage::Hotkey(HotkeyMessage::PlayPause) => {
                self.player.play_pause();
            }
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

        let player = Player::new(sender.clone());

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
    fn register_hotkeys(
        sender: Arc<UnboundedSender<UampMessage>>,
    ) -> Result<GlobalHotKeyManager> {
        let hotkey_mgr = GlobalHotKeyManager::new()?;

        let play_pause = HotKey::new(
            Some(hotkey::Modifiers::CONTROL | hotkey::Modifiers::ALT),
            Code::Home,
        );
        let play_pause_id = play_pause.id();

        hotkey_mgr.register(play_pause)?;

        GlobalHotKeyEvent::set_event_handler(Some(
            move |e: GlobalHotKeyEvent| {
                match e.id {
                    id if id == play_pause_id => {
                        _ = sender.send(UampMessage::Hotkey(
                            HotkeyMessage::PlayPause,
                        ));
                    }
                    _ => {}
                };
            },
        ));

        Ok(hotkey_mgr)
    }

    fn start_server() -> Result<TcpListener> {
        Ok(TcpListener::bind(format!("127.0.0.1:{}", default_port()))?)
    }
}

impl From<messenger::Control> for UampMessage {
    fn from(value: messenger::Control) -> Self {
        match value {
            messenger::Control::PlayPause => UampMessage::PlayPause,
        }
    }
}
