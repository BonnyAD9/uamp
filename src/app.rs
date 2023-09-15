use std::{
    cell::Cell,
    net::TcpListener,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use iced::{executor, window, Application};
use iced_core::Event;
use log::{error, warn};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{
    config::{app_id, Config},
    core::{
        messenger::{self, Messenger},
        msg::{ComMsg, ControlMsg, Msg},
        Error, Result,
    },
    gui::{
        app::GuiState,
        theme::Theme,
        wid::{Command, Element, Subscription},
        GuiMessage, WinMessage,
    },
    hotkeys::{Action, Hotkey, HotkeyMgr},
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
    pub reciever: Cell<Option<UnboundedReceiver<Msg>>>,

    /// The visual style/theme of the app
    pub theme: Theme,
    /// The state of gui
    pub gui: GuiState,

    /// hotkey manager
    pub hotkey_mgr: HotkeyMgr,
    /// The server listener
    pub listener: Cell<Option<TcpListener>>,

    /// Messages that can be processed only if there are no running processes
    pub pending_close: bool,

    /// When was last save
    pub last_save: Instant,
}

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

impl UampApp {
    /// Saves all the data that is saved by uamp
    pub fn save_all(&mut self) {
        match self
            .library
            .start_to_default_json(&self.config, self.sender.clone())
        {
            Err(Error::InvalidOperation(_)) => {}
            Err(e) => error!("Failed to start library save: {e}"),
            _ => {}
        }
        if let Err(e) = self.player.to_default_json(&self.config) {
            error!("Failed to save play state: {e}");
        }
        if let Err(e) = self.config.to_default_json() {
            error!("Failed to save config: {e}");
        }
        if let Err(e) = self.gui.to_default_json(&self.config) {
            error!("Failed to save gui state: {e}");
        }

        self.last_save = Instant::now();
    }
}

impl Application for UampApp {
    type Executor = executor::Default;
    type Flags = (Config, GuiState);
    type Message = Msg;
    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Self, Command) {
        (UampApp::new(flags.0, flags.1), Command::none())
    }

    fn update(&mut self, message: Self::Message) -> Command {
        let com = match message {
            Msg::PlaySong(index, songs) => {
                self.player.play_playlist(
                    &mut self.library,
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
            Msg::Delegate(d) => d.update(self),
            Msg::WindowChange(msg) => self.win_event(msg),
            Msg::Config(msg) => self.config_event(msg),
            Msg::Init => self.init(),
        };

        // The recursion that follows is all tail call recursion that will be
        // optimized so that there is no recursion

        let com = match com {
            ComMsg::Command(com) => com,
            ComMsg::Msg(msg) => return self.update(msg),
        };

        if self.pending_close {
            if !self.library.any_process() {
                self.pending_close = false;
                return self.update(Msg::Control(ControlMsg::Close));
            }
        }

        if self
            .config
            .save_timeout()
            .map(|t| Instant::now() - self.last_save >= t.0)
            .unwrap_or_default()
        {
            self.library.any_process();
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

    fn subscription(&self) -> Subscription {
        if self.config.enable_server() {
            iced::subscription::Subscription::batch([
                self.reciever_subscription(),
                self.server_subscription(),
                self.events_subscription(),
                self.clock_subscription(self.config.tick_length().0),
            ])
        } else {
            iced::subscription::Subscription::batch([
                self.reciever_subscription(),
                self.events_subscription(),
                self.clock_subscription(self.config.tick_length().0),
            ])
        }
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl UampApp {
    fn new(conf: Config, gui: GuiState) -> Self {
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

        let listener = if conf.enable_server() {
            match Self::start_server(&conf) {
                Ok(l) => Cell::new(Some(l)),
                Err(e) => {
                    error!("Failed to start the server: {e}");
                    Cell::new(None)
                }
            }
        } else {
            Cell::new(None)
        };

        if let Err(e) = sender.send(Msg::Init) {
            error!("Failed to send init message: {e}")
        }

        UampApp {
            config: conf,
            library: lib,
            player,

            sender,
            reciever: Cell::new(Some(reciever)),

            theme: Theme::default(),
            gui,

            hotkey_mgr: HotkeyMgr::new(),
            listener,

            pending_close: false,

            last_save: Instant::now(),
        }
    }

    fn init(&mut self) -> ComMsg {
        if self.config.register_global_hotkeys() {
            if let Err(e) = self.hotkey_mgr.init(
                self.sender.clone(),
                self.config.global_hotkeys().iter().filter_map(|(h, a)| {
                    let h = match h.parse::<Hotkey>() {
                        Ok(h) => h,
                        Err(e) => {
                            error!("Failed to parse hotkey: {e}");
                            return None;
                        }
                    };
                    let a = match a.parse::<Action>() {
                        Ok(a) => a,
                        Err(e) => {
                            error!("Failed to parse hotkey action: {e}");
                            return None;
                        }
                    };

                    Some((h, a))
                }),
            ) {
                error!("Failed to initialize hotkeys: {e}");
            }
        }

        ComMsg::none()
    }

    /// Starts the tcp server
    fn start_server(conf: &Config) -> Result<TcpListener> {
        Ok(TcpListener::bind(format!(
            "{}:{}",
            conf.server_address(),
            conf.port()
        ))?)
    }

    fn reciever_subscription(&self) -> Subscription {
        let id = app_id() + " async msg";
        if let Some(r) = self.reciever.take() {
            iced::subscription::unfold(id, r, |mut reciever| async {
                let msg = reciever.recv().await.unwrap();
                (msg, reciever)
            })
        } else {
            self.fake_sub(id)
        }
    }

    fn server_subscription(&self) -> Subscription {
        let id = app_id() + " server";
        if let Some(l) = self.listener.take() {
            iced::subscription::unfold(id, l, |listener| async {
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

                    let (response, msg) = Self::message_event(rec, &stream.0);
                    if let Some(r) = response {
                        if let Err(e) = msgr.send(r) {
                            warn!("Failed to send response {e}");
                        }
                    }

                    if let Some(msg) = msg {
                        break (msg, listener);
                    } else {
                        continue;
                    }
                }
            })
        } else {
            self.fake_sub(id)
        }
    }

    fn events_subscription(&self) -> Subscription {
        iced::subscription::events_with(|e, _| match e {
            Event::Window(window::Event::CloseRequested) => {
                Some(Msg::Control(ControlMsg::Close))
            }
            Event::Window(window::Event::Moved { x, y }) => {
                Some(Msg::WindowChange(WinMessage::Position(x, y)))
            }
            Event::Window(window::Event::Resized { width, height }) => {
                Some(Msg::WindowChange(WinMessage::Size(width, height)))
            }
            _ => None,
        })
    }

    fn clock_subscription(&self, tick: Duration) -> Subscription {
        iced::subscription::unfold(
            app_id() + " tick",
            Instant::now(),
            move |i| async move {
                let now = Instant::now();
                let dif = now - i;
                if dif < tick {
                    thread::sleep(tick - dif);
                }
                (Msg::Gui(GuiMessage::Tick), i + tick)
            },
        )
    }

    fn fake_sub(&self, id: String) -> Subscription {
        iced::subscription::unfold(id, (), |_| async {
            loop {
                println!("hi")
            }
        })
    }
}
