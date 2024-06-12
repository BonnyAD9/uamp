use serde::{Deserialize, Serialize};

use crate::core::msg::{AnyControlMsg, PlayMsg};

use super::{Error, ErrorKind, Info, Request};

/// Messages passed between uamp instances.
#[derive(Debug, Serialize, Deserialize)]
pub enum MsgMessage {
    /// An error occured.
    Error(Error),
    /// Play something.
    Play(PlayMsg),
    /// todo.
    Request(Request),
    /// Send simple action to be done.
    Control(AnyControlMsg),
    /// Response to info request.
    Info(Box<Info>),
    /// Message indicating success.
    Success,
    /// Stop the server
    Stop,
    /// Try to connect to the uamp.
    Ping,
}

impl MsgMessage {
    /// Creates new error with a default message for its type
    pub fn new_error(typ: ErrorKind) -> Self {
        match typ {
            ErrorKind::DeserializeFailed => Self::Error(Error::new(
                typ,
                "Failed to deserialize the incoming message".to_owned(),
            )),
            ErrorKind::ExpectedRequest => Self::Error(Error::new(
                typ,
                "Expected request message".to_owned(),
            )),
            ErrorKind::ExpectedControl => Self::Error(Error::new(
                typ,
                "Expected control message".to_owned(),
            )),
            ErrorKind::ExpectedInfo => Self::Error(Error::new(
                typ,
                "Expected info message".to_owned(),
            )),
            ErrorKind::ExpectedRequestOrControl => Self::Error(Error::new(
                typ,
                "Expected request or control message".to_owned(),
            )),
            ErrorKind::InternalError => Self::Error(Error::new(
                typ,
                "Error occured when trying to fulfill request.".to_owned(),
            )),
        }
    }
}
