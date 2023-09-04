mod message;
mod messenger;

pub mod msg {
    pub use super::message::*;
}

pub use messenger::*;

use crate::app::UampApp;
use msg::Message;

use self::msg::ErrorType;

use super::msg::Msg;

impl UampApp {
    /// Translates [`Message`] to [`UampMessage`].
    ///
    /// Returns message that should be sent as a response and the translated
    /// [`UampMessage`] if it should produce one.
    pub fn message_event(msg: Message) -> (Message, Option<Msg>) {
        let msg = if let Some(msg) = msg.control() {
            msg
        } else {
            return (Message::new_error(ErrorType::ExpectedControl), None);
        };

        (Message::Success, Some(Msg::Control(msg)))
    }
}
