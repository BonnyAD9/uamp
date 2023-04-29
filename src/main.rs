use eyre::Result;
use iced::{
    executor,
    widget::{button, column, text},
    Application, Command, Element, Settings, Theme,
};

fn main() -> Result<()> {
    Counter::run(Settings::default())?;
    Ok(())
}

struct Counter {
    value: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    IncrementPressed,
    DecrementPressed,
}

impl Application for Counter {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn view(&self) -> Element<Self::Message> {
        column![
            button("+").on_press(Message::IncrementPressed),
            text(self.value).size(50),
            button("-").on_press(Message::DecrementPressed)
        ]
        .into()
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::IncrementPressed => self.value += 1,
            Message::DecrementPressed => self.value -= 1,
        }
        Command::none()
    }

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (Counter { value: 0 }, Command::none())
    }

    fn title(&self) -> String {
        "uamp".to_owned()
    }
}
