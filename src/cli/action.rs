use crate::core::messenger::msg;

use super::RunInfo;

/// Action that can be done with cli
pub enum Action {
    /// Sends the given message
    Message(Vec<msg::Message>),
    RunDetached(RunInfo),
}
