use iced::{executor, widget, Application, Command, Element, Length, Theme};
use iced_native::widget::{column, scrollable};

use crate::{
    config::Config, library::Library, player::Player, wrap_box::wrap_box,
};

pub struct UampApp {
    config: Config,
    library: Library,
    player: Player,
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
                _ = self.player.play(&self.library, id)
            }
            UampMessage::PlayPause => {
                self.player.play_pause();
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
                    .height(Length::Fixed(30.))
                    .into()
                })
                .collect(),
        )
        .item_height(30)
        .spacing_y(5);

        let now_playing = widget::button(widget::text("Play/Pause"))
            .on_press(UampMessage::PlayPause);

        widget::column![
            list.height(Length::Fill),
            now_playing.height(Length::Fixed(30.))
        ]
        .into()

        //list.into()
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
