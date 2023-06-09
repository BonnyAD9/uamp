use iced::{executor, widget, Application, Command, Element, Length, Theme};

use crate::{
    config::Config,
    fancy_widgets::{icons, wrap_box::wrap_box},
    library::Library,
    player::Player,
};

use self::PlayState::{Paused, Playing, Stopped};

pub struct UampApp {
    config: Config,
    library: Library,
    player: Player,

    now_playing: PlayState,
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

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        _ = flags;
        (UampApp::default(), Command::none())
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
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

    fn view(&self) -> Element<Self::Message> {
        _ = self.config;
        let mut c = 0;
        let list = wrap_box(
            self.library
                .iter()
                .map(|s| {
                    c += 1;
                    widget::button(widget::text(format!(
                        "{} - {}",
                        s.artist(),
                        s.title()
                    )))
                    .on_press(UampMessage::PlaySong(c - 1))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                })
                .collect(),
        )
        .item_height(30)
        .spacing_y(5);

        let now_playing =
            widget::button(widget::svg(if self.now_playing.is_playing() {
                icons::PAUSE
            } else {
                icons::PLAY
            }))
            .on_press(UampMessage::PlayPause)
            .width(Length::Fixed(30.))
            .height(Length::Fixed(30.));

        widget::column![list.height(Length::Fill), now_playing,].into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
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
