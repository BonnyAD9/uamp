use crate::core::{messenger::msg, msg::ControlMsg};

/// Action that can be done with cli
pub enum Action {
    /// Sends the given message
    Message(msg::Message),
}

impl Action {
    /// Creates control message
    pub fn control(msg: ControlMsg) -> Self {
        Self::Message(msg::Message::Control(msg))
    }
}
