use std::{cell::RefCell, net::TcpListener, sync::Arc, time::Instant};

use global_hotkey::{
    hotkey::{self, Code, HotKey},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use iced::{executor, window, Application};
use iced_core::Event;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{
    config::{app_id, default_port, Config},
    err::{Error, Result},
    library::{Library, LibraryMessage, SongId},
    messenger::{self, Messenger},
    player::{Player, PlayerMessage},
    theme::Theme,
    uamp_gui::{self, GuiState},
    wid::{Command, Element},
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
    pub sender: Arc<UnboundedSender<UampMessage>>,
    /// Reciever of the async messages
    pub reciever: RefCell<Option<UnboundedReceiver<UampMessage>>>,

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

/// Event messages in uamp
#[allow(missing_debug_implementations)]
#[derive(Clone, Debug)]
pub enum UampMessage {
    /// Play song song at the given index in the playlist
    PlaySong(usize, Arc<[SongId]>),
    /// Some simple messages
    Control(ControlMsg),
    /// Gui messages handled by the gui
    Gui(uamp_gui::Message),
    /// Player messges handled by the player
    Player(PlayerMessage),
    /// Library messages handled by the library
    Library(LibraryMessage),
}

/// only simple messages that can be safely send across threads and copied
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ControlMsg {
    /// Toggle between play/pause
    PlayPause,
    /// Jump to the next song
    NextSong,
    /// Jump to the previous song
    PrevSong,
    /// Set the volume
    SetVolume(f32),
    /// Increase the volume
    VolumeUp,
    /// Decrease the volume
    VolumeDown,
    /// Toggle the mute control
    ToggleMute,
    /// Shuffle the current playlist
    Shuffle,
    /// Jump to the given index in the playlist
    PlaylistJump(usize),
    /// Exit the app
    Close,
    /// Search for new songs
    FindSongs,
}

impl Application for UampApp {
    type Executor = executor::Default;
    type Flags = ();
    type Message = UampMessage;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command) {
        (UampApp::default(), Command::none())
    }

    fn update(&mut self, message: Self::Message) -> Command {
        let com = match message {
            UampMessage::PlaySong(index, songs) => {
                self.player.play_playlist(
                    &self.library,
                    songs,
                    Some(index),
                    true,
                );
                Command::none()
            }
            UampMessage::Control(msg) => self.control_event(msg),
            UampMessage::Gui(msg) => self.gui_event(msg),
            UampMessage::Player(msg) => self.player.event(&self.library, msg),
            UampMessage::Library(msg) => self.library.event(msg, &self.config),
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
                                warn!("Failed to recieve message: {e}");
                                if let Err(e) = msgr.send(messenger::error(
                                    messenger::ErrorType::DeserializeFailed,
                                    e.to_string(),
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

        let (sender, reciever) = mpsc::unbounded_channel::<UampMessage>();
        let sender = Arc::new(sender);

        if conf.update_library_on_start() {
            if let Err(e) = lib.start_get_new_songs(&conf, sender.clone()) {
                error!("Failed to start library load: {e}");
            }
        }

        let mut player = Player::from_config(sender.clone(), &conf);
        player.fade_play_pause(conf.fade_play_pause());

        let hotkey_mgr = if conf.register_global_hotkeys() {
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

            last_save: Instant::now(),
        }
    }
}

impl UampApp {
    /// handles the control events
    fn control_event(&mut self, msg: ControlMsg) -> Command {
        match msg {
            ControlMsg::PlayPause => {
                self.player.play_pause(&self.library);
            }
            ControlMsg::NextSong => self.player.play_next(&self.library),
            ControlMsg::PrevSong => self.player.play_prev(&self.library),
            ControlMsg::Close => {
                self.save_all();
                return window::close();
            }
            ControlMsg::Shuffle => self.player.shuffle(),
            ControlMsg::SetVolume(v) => {
                self.player.set_volume(v.clamp(0., 1.))
            }
            ControlMsg::VolumeUp => self.player.set_volume(
                (self.player.volume() + self.config.volume_jump())
                    .clamp(0., 1.),
            ),
            ControlMsg::VolumeDown => self.player.set_volume(
                (self.player.volume() - self.config.volume_jump())
                    .clamp(0., 1.),
            ),
            ControlMsg::PlaylistJump(i) => {
                self.player
                    .play_at(&self.library, i, self.player.is_playing())
            }
            ControlMsg::ToggleMute => self.player.toggle_mute(),
            ControlMsg::FindSongs => {
                match self
                    .library
                    .start_get_new_songs(&self.config, self.sender.clone())
                {
                    Err(e) if matches!(e, Error::InvalidOperation(_)) => {
                        info!("Cannot load new songs: {e}")
                    }
                    Err(e) => error!("Cannot load new songs: {e}"),
                    _ => {}
                }
            }
        };

        Command::none()
    }

    /// Saves all the data that is saved by uamp
    fn save_all(&mut self) {
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

    /// Registers hotkeys
    fn register_hotkeys(
        sender: Arc<UnboundedSender<UampMessage>>,
    ) -> Result<GlobalHotKeyManager> {
        macro_rules! hotkey {
            ($first:ident + $second:ident $(-$rest:ident)*) => {
                hotkey!($second - $first $(-$rest)*)
            };
            (
                $first:ident
                + $second:ident
                $(+$tail:ident)+
                $(-$rest:ident)*
            ) => {
                hotkey!($second $(+$tail)+ - $first $(-$rest)*)
            };
            ($key:ident - $first:ident $(-$tail:ident)*) => {{
                let key = HotKey::new(Some(
                    hotkey::Modifiers::$first $(| hotkey::Modifiers::$tail)*),
                    Code::$key
                );
                let id = key.id();
                (key, id)
            }};
        }

        macro_rules! make_hotkeys {
            (
                $($key:ident $(+$mods:ident)+ -> $name:ident : $action:ident),+
                $(,)?
            ) => {{
                let hotkey_mgr = GlobalHotKeyManager::new()?;

                $(let $name = hotkey!($key $(+$mods)+);)+

                hotkey_mgr.register_all(&[
                    $($name.0),+
                ])?;

                GlobalHotKeyEvent::set_event_handler(Some(
                    move |e: GlobalHotKeyEvent| {
                        match e.id {$(
                            id if id == $name.1 => {
                                if let Err(e) = sender.send(
                                    UampMessage::Control(ControlMsg::$action)
                                ) {
                                    error!(
                                        "Failed to send hotkey message: {e}"
                                    );
                                }
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

    /// Starts the tcp server
    fn start_server() -> Result<TcpListener> {
        Ok(TcpListener::bind(format!("127.0.0.1:{}", default_port()))?)
    }
}
