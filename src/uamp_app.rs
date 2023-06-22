use std::sync::Arc;

use iced::{executor, Application};
use iced_native::{event::Status, Clipboard, Event, Point};

use crate::{
    config::Config,
    library::Library,
    player::Player,
    theme::Theme,
    uamp_gui::{self, GuiState},
    wid::{Command, Element, IteratorFn},
};

pub struct UampApp {
    pub config: Config,
    pub library: Library,
    pub player: Player,

    pub theme: Theme,

    pub gui: GuiState,

    pub now_playing: PlayState,
}

#[allow(missing_debug_implementations)]
#[derive(Clone, Debug)]
pub enum UampMessage {
    PlaySong(usize, Arc<dyn IteratorFn>),
    PlayPause,
    Gui(uamp_gui::Message),
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
                self.now_playing.play_new(
                    &self.library,
                    songs.as_ref(),
                    index,
                );
                _ = self.player.play(
                    &self.library,
                    self.now_playing.now_playing().unwrap(),
                );
            }
            UampMessage::PlayPause => {
                self.player.play_pause();
                self.now_playing.play_pause();
            }
            UampMessage::Gui(msg) => return self.gui_event(msg),
        };
        Command::none()
    }

    fn title(&self) -> String {
        "uamp".to_owned()
    }

    fn view(&self) -> Element {
        self.gui()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
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
        let player = Player::try_new().unwrap();

        UampApp {
            config: conf,
            library: lib,
            player,
            theme: Theme::default(),
            gui: GuiState::default(),
            now_playing: PlayState::default(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Playback {
    Stopped,
    Playing(usize),
    Paused(usize),
}

pub struct PlayState {
    playback: Playback,
    playlist: Vec<usize>,
}

impl PlayState {
    fn new() -> Self {
        PlayState {
            playback: Playback::Stopped,
            playlist: Vec::new(),
        }
    }

    pub fn play_pause(&mut self) {
        match self.playback {
            Playback::Stopped => {}
            Playback::Playing(id) => {
                self.playback = Playback::Paused(id);
            }
            Playback::Paused(id) => {
                self.playback = Playback::Playing(id);
            }
        };
    }

    pub fn play_new<FI>(&mut self, library: &Library, songs: &FI, index: usize)
    where
        FI: IteratorFn + ?Sized,
    {
        self.playlist = songs(library).collect();
        self.playback = Playback::Playing(index);
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.playback, Playback::Playing(_))
    }

    pub fn now_playing(&self) -> Option<usize> {
        match self.playback {
            Playback::Stopped => None,
            Playback::Paused(s) => Some(self.playlist[s]),
            Playback::Playing(s) => Some(self.playlist[s]),
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
