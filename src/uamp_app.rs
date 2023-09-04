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
    /// Toggle/set between play/pause, [`None`] to toggle, [`Some`] to set
    PlayPause(Option<bool>),
    /// Jump to the Nth next song
    NextSong(usize),
    /// Jump to the Nth previous song
    PrevSong(usize),
    /// Set the volume
    SetVolume(f32),
    /// Increase the volume by `vol_jump * .0`
    VolumeUp(f32),
    /// Decrease the volume by `vol_jump * .0`
    VolumeDown(f32),
    /// Toggle/set the mute control, [`None`] to toggle, [`Some`] to set
    Mute(Option<bool>),
    /// Shuffle the current playlist
    Shuffle,
    /// Jump to the given index in the playlist
    PlaylistJump(usize),
    /// Exit the app
    Close,
    /// Search for new songs
    LoadNewSongs,
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
        let mut conf = Config::from_default_json();

        let mut lib = Library::from_config(&conf);

        let (sender, reciever) = mpsc::unbounded_channel::<UampMessage>();
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
}

impl UampApp {
    /// handles the control events
    fn control_event(&mut self, msg: ControlMsg) -> Command {
        match msg {
            ControlMsg::PlayPause(p) => {
                self.player.play_pause(
                    &self.library,
                    p.unwrap_or(!self.player.is_playing()),
                );
            }
            ControlMsg::NextSong(n) => self.player.play_next(&self.library, n),
            ControlMsg::PrevSong(n) => self.player.play_prev(&self.library, n),
            ControlMsg::Close => {
                self.save_all();
                return window::close();
            }
            ControlMsg::Shuffle => self.player.shuffle(),
            ControlMsg::SetVolume(v) => {
                self.player.set_volume(v.clamp(0., 1.))
            }
            ControlMsg::VolumeUp(m) => self.player.set_volume(
                (self.player.volume() + self.config.volume_jump() * m)
                    .clamp(0., 1.),
            ),
            ControlMsg::VolumeDown(m) => self.player.set_volume(
                (self.player.volume() - self.config.volume_jump() * m)
                    .clamp(0., 1.),
            ),
            ControlMsg::PlaylistJump(i) => {
                self.player
                    .play_at(&self.library, i, self.player.is_playing())
            }
            ControlMsg::Mute(b) => {
                self.player.set_mute(b.unwrap_or(!self.player.mute()))
            }
            ControlMsg::LoadNewSongs => {
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

    /// Starts the tcp server
    fn start_server() -> Result<TcpListener> {
        Ok(TcpListener::bind(format!("127.0.0.1:{}", default_port()))?)
    }
}
