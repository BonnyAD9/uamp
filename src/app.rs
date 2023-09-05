use std::{
    cell::RefCell,
    net::TcpListener,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use global_hotkey::GlobalHotKeyManager;
use iced::{executor, window, Application};
use iced_core::Event;
use log::{error, warn};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{
    config::{app_id, default_port, Config},
    core::{
        messenger::{self, Messenger},
        msg::{ComMsg, ControlMsg, Msg},
        Result,
    },
    gui::{
        app::GuiState,
        theme::Theme,
        wid::{Command, Element},
        GuiMessage,
    },
    library::Library,
    player::Player,
};

/// The uamp app state
pub struct UampApp {
    /// The configuration
    pub config: Config,
    /// The song library
    pub library: Library,
    /// The song player
    pub player: Player,

    /// Sender for async messages to be synchronized with the main message
    /// handler
    pub sender: Arc<UnboundedSender<Msg>>,
    /// Reciever of the async messages
    pub reciever: RefCell<Option<UnboundedReceiver<Msg>>>,

    /// The visual style/theme of the app
    pub theme: Theme,
    /// The state of gui
    pub gui: GuiState,

    /// hotkey manager
    pub hotkey_mgr: Option<GlobalHotKeyManager>,
    /// The server listener
    pub listener: RefCell<Option<TcpListener>>,

    /// When was last save
    pub last_save: Instant,
}


//===========================================================================//
//                                   Public                                  //
//===========================================================================//

impl UampApp {
    /// Saves all the data that is saved by uamp
    pub fn save_all(&mut self) {
        if let Err(e) = self.library.to_default_json(&self.config) {
            error!("Failed to save library: {e}");
        }
        if let Err(e) = self.player.to_default_json(&self.config) {
            error!("Failed to save play state: {e}");
        }
        if let Err(e) = self.config.to_default_json() {
            error!("Failed to save config: {e}");
        }

        self.last_save = Instant::now();
    }
}

impl Application for UampApp {
    type Executor = executor::Default;
    type Flags = Config;
    type Message = Msg;
    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Self, Command) {
        (UampApp::new(flags), Command::none())
    }

    fn update(&mut self, message: Self::Message) -> Command {
        let com = match message {
            Msg::PlaySong(index, songs) => {
                self.player.play_playlist(
                    &self.library,
                    songs,
                    Some(index),
                    true,
                );
                ComMsg::tick()
            }
            Msg::Control(msg) => self.control_event(msg),
            Msg::Gui(msg) => self.gui_event(msg),
            Msg::Player(msg) => self.player_event(msg),
            Msg::Library(msg) => self.library_event(msg),
        };

        let com = match com {
            ComMsg::Command(com) => com,
            ComMsg::Msg(msg) => return self.update(msg),
        };

        if self
            .config
            .save_timeout()
            .map(|t| (Instant::now() - self.last_save).as_secs_f32() >= t)
            .unwrap_or_default()
        {
            self.save_all()
        }

        com
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
        let tick_len = Duration::from_secs_f32(self.config.tick_length());
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

                        let rec = match rec {
                            Ok(m) => m,
                            Err(e) => {
                                warn!("Failed to recieve message: {e}");
                                if let Err(e) = msgr.send(messenger::msg::Message::Error(
                                    messenger::msg::Error::new(
                                    messenger::msg::ErrorType::DeserializeFailed,
                                    e.to_string(),
                                )
                                )) {
                                    warn!("Failed to send error message: {e}");
                                }
                                continue;
                            }
                        };

                        let (response, msg) = Self::message_event(rec);
                        if let Err(e) = msgr.send(response) {
                            warn!("Failed to send response {e}");
                        }

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
                    Some(Msg::Control(ControlMsg::Close))
                }
                _ => None,
            }),
            // ticks clock and ensures that errors don't accumulate
            iced::subscription::unfold(
                app_id() + " tick",
                Instant::now(),
                move |i| async move {
                    let now = Instant::now();
                    let dif = now - i;
                    if dif < tick_len {
                        thread::sleep(tick_len - dif);
                    }
                    (Msg::Gui(GuiMessage::Tick), i + tick_len)
                },
            ),
        ])
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl UampApp {
    fn new(mut conf: Config) -> Self {
        let mut lib = Library::from_config(&conf);

        let (sender, reciever) = mpsc::unbounded_channel::<Msg>();
        let sender = Arc::new(sender);

        if conf.update_library_on_start() {
            if let Err(e) = lib.start_get_new_songs(&conf, sender.clone()) {
                error!("Failed to start library load: {e}");
            }
        }

        let mut player = Player::from_config(sender.clone(), &conf);
        player.load_config(&conf);

        let hotkey_mgr = match conf.register_hotkeys(sender.clone()) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to register global hotkeys: {e}");
                None
            }
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

            last_save: Instant::now(),
        }
    }

    /// Starts the tcp server
    fn start_server() -> Result<TcpListener> {
        Ok(TcpListener::bind(format!("127.0.0.1:{}", default_port()))?)
    }
}
