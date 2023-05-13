use iced::{executor, widget, Application, Command, Element, Theme};

use crate::{config::Config, library::Library, player::Player};

pub struct UampApp {
    config: Config,
    library: Library,
    player: Player,
}

#[derive(Debug, Clone, Copy)]
pub enum UampMessage {
    PlaySong(usize),
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
                _ = self.player.play(&self.library, id)
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
        widget::scrollable(widget::column(
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
                    .into()
                })
                .collect(),
        ))
        .into()
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
        }
    }
}
