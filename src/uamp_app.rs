use std::{cell::RefCell, sync::Arc};

use eyre::Result;
use global_hotkey::{
    hotkey::{self, Code, HotKey},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use iced::{executor, Application};
use iced_core::{event::Status, Clipboard, Event, Point};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{
    config::{app_id, Config},
    library::Library,
    player::Player,
    theme::Theme,
    uamp_gui::{self, GuiState},
    wid::{Command, Element},
};

pub struct UampApp {
    pub config: Config,
    pub library: Library,
    pub player: Player,

    pub sender: Arc<UnboundedSender<AsyncMessage>>,
    pub reciever: RefCell<Option<UnboundedReceiver<AsyncMessage>>>,

    pub theme: Theme,
    pub gui: GuiState,

    pub now_playing: PlayState,

    pub hotkey_mgr: Option<GlobalHotKeyManager>,
}

#[allow(missing_debug_implementations)]
#[derive(Clone, Debug)]
pub enum UampMessage {
    PlaySong(usize, Arc<[usize]>),
    PlayPause,
    Gui(uamp_gui::Message),
    Async(AsyncMessage),
}

#[derive(Clone, Debug)]
pub enum AsyncMessage {
    SongEnd,
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
                self.now_playing.play_new(songs, Some(index));
                _ = self.player.play(
                    &self.library,
                    self.now_playing.now_playing().unwrap(),
                );
            }
            UampMessage::PlayPause
            | UampMessage::Async(AsyncMessage::PlayPause) => {
                _ = self.player.play_pause(self.now_playing.play_pause());
            }
            UampMessage::Gui(msg) => return self.gui_event(msg),
            UampMessage::Async(msg) => return self.async_event(msg),
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
                    (UampMessage::Async(msg), reciever)
                },
            ),
            /*iced::subscription::events_with(|e, s| {
                match e {
                    Event::Keyboard(k) => match k {
                        iced_core::keyboard::Event::KeyPressed { key_code, modifiers } => {
                            if key_code == KeyCode::Home && modifiers.control() && modifiers.alt() {
                                Some(UampMessage::PlayPause)
                            } else {
                                None
                            }
                        },
                        _ => None,
                    },
                    _ => None
                }
            })*/
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

        // XXX: try to avoid unwrap
        let mut player = Player::try_new().unwrap();

        let (sender, reciever) = mpsc::unbounded_channel::<AsyncMessage>();
        let sender = Arc::new(sender);

        let send_clone = sender.clone();
        _ = player.on_song_end(move || {
            _ = send_clone.send(AsyncMessage::SongEnd);
        });

        let hotkey_mgr = if conf.register_global_hotkes {
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

            now_playing: PlayState::default(),

            // XXX: avoid unwrap
            hotkey_mgr,
        }
    }
}

impl UampApp {
    fn async_event(&mut self, msg: AsyncMessage) -> Command {
        match msg {
            AsyncMessage::SongEnd => {
                if let Some(s) = self.now_playing.play_next() {
                    _ = self.player.play(&self.library, s);
                }
            }
            _ => {}
        }

        Command::none()
    }

    fn register_hotkeys(
        sender: Arc<UnboundedSender<AsyncMessage>>,
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
                        _ = sender.send(AsyncMessage::PlayPause);
                    }
                    _ => {}
                };
            },
        ));

        Ok(hotkey_mgr)
    }
}

#[derive(Clone, Copy)]
pub enum Playback {
    Stopped,
    Playing,
    Paused,
}

pub struct PlayState {
    playback: Playback,
    playlist: Arc<[usize]>,
    current: Option<usize>,
}

impl PlayState {
    fn new() -> Self {
        PlayState {
            playback: Playback::Stopped,
            playlist: [][..].into(),
            current: None,
        }
    }

    pub fn play_pause(&mut self) -> bool {
        match self.playback {
            Playback::Stopped => false,
            Playback::Playing => {
                self.playback = Playback::Paused;
                false
            }
            Playback::Paused => {
                self.playback = Playback::Playing;
                true
            }
        }
    }

    pub fn play_new(&mut self, songs: Arc<[usize]>, index: Option<usize>) {
        self.playlist = songs;
        self.playback = Playback::Playing;
        self.current = index;
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.playback, Playback::Playing)
    }

    pub fn now_playing(&self) -> Option<usize> {
        self.current
    }

    pub fn play_next(&mut self) -> Option<usize> {
        match self.playback {
            Playback::Stopped => None,
            Playback::Playing => {
                if let Some(mut cur) = self.current {
                    cur += 1;
                    if cur == self.playlist.len() {
                        self.playback = Playback::Stopped;
                        self.current = None;
                        None
                    } else {
                        self.current = Some(cur);
                        Some(self.playlist[cur])
                    }
                } else {
                    None
                }
            }
            Playback::Paused => todo!(),
        }
    }
}

impl Default for PlayState {
    fn default() -> Self {
        PlayState::new()
    }
}

impl UampApp {
    pub fn events(
        &self,
        event: Event,
        _cursor: Point,
        _clipboard: &mut dyn Clipboard,
    ) -> (Option<UampMessage>, Status) {
        println!("{event:?}");
        match event {
            _ => (None, Status::Ignored),
        }
    }
}
