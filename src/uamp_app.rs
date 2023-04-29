use iced::{executor, widget::text, Application, Command, Element, Theme};

pub struct UampApp {}

#[derive(Debug, Clone, Copy)]
pub enum UampMessage {}

impl Application for UampApp {
    type Executor = executor::Default;
    type Flags = ();
    type Message = UampMessage;
    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        _ = flags;
        (UampApp {}, Command::none())
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        _ = message;
        Command::none()
    }

    fn title(&self) -> String {
        "uamp".to_owned()
    }

    fn view(&self) -> Element<Self::Message> {
        text("your music will be here :)").into()
    }
}
