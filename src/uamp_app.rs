use iced::{clipboard, executor, window, Application};
use iced_native::{event::Status, Clipboard, Event, Point};

use crate::{
    config::Config,
    library::Library,
    player::Player,
    theme::Theme,
    wid::{Command, Element},
};

use self::PlayState::{Paused, Playing, Stopped};

pub struct UampApp {
    pub config: Config,
    pub library: Library,
    pub player: Player,

    pub theme: Theme,

    pub now_playing: PlayState,
}

#[derive(Debug, Clone, Copy)]
pub enum UampMessage {
    PlaySong(usize),
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
            UampMessage::PlaySong(id) => {
                _ = self.player.play(&self.library, id);
                self.now_playing = Playing(id);
            }
            UampMessage::PlayPause => {
                self.player.play_pause();
                self.now_playing.play_pause();
            }
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
        let conf = Config::default();
        let lib = Library::from_config(&conf).unwrap_or_default();

        // XXX: try to avoid unwrap
        let player = Player::try_new().unwrap();

        UampApp {
            config: conf,
            library: lib,
            player,
            theme: Theme::default(),
            now_playing: Stopped,
        }
    }
}

#[derive(Clone, Copy)]
pub enum PlayState {
    Stopped,
    Playing(usize),
    Paused(usize),
}

impl PlayState {
    pub fn play_pause(&mut self) -> Self {
        match *self {
            Stopped => {}
            Playing(id) => {
                *self = Paused(id);
            }
            Paused(id) => {
                *self = Playing(id);
            }
        };
        *self
    }

    pub fn is_playing(&self) -> bool {
        matches!(self, Playing(_))
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
