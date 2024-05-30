use crate::core::messenger::msg;

/// Action that can be done with cli
pub enum Action {
    /// Sends the given message
    Message(Vec<msg::Message>),
}
