use iced::{executor, widget, Application, Command, Element, Theme};

use crate::{config::Config, library::Library};

pub struct UampApp {
    config: Config,
    library: Library,
}

#[derive(Debug, Clone, Copy)]
pub enum UampMessage {}

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
        _ = message;
        Command::none()
    }

    fn title(&self) -> String {
        "uamp".to_owned()
    }

    fn view(&self) -> Element<Self::Message> {
        _ = self.config;
        widget::scrollable(widget::column(
            self.library
                .songs
                .iter()
                .map(|s| widget::text(s.title.to_string()).into())
                .collect(),
        ))
        .into()
    }
}

impl Default for UampApp {
    fn default() -> Self {
        let conf = Config::default();
        let lib = Library::from_config(&conf).unwrap_or_default();

        UampApp {
            config: conf,
            library: lib,
        }
    }
}
