use crate::core::{messenger::msg, msg::ControlMsg};

/// Action that can be done with cli
pub enum Action {
    Message(msg::Message),
}

impl Action {
    pub fn control(msg: ControlMsg) -> Self {
        Self::Message(msg::Message::Control(msg))
    }
}
