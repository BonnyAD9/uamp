use serde::{Deserialize, Serialize};

use crate::core::{AnyControlMsg, ControlMsg, DataControlMsg, PlayMsg};

use super::{DataResponse, Error, ErrorKind, Request};

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
    Data(DataResponse),
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
            ErrorKind::DeserializeFailed => Error::new(
                typ,
                "Failed to deserialize the incoming message"
                    .to_owned()
                    .into(),
            )
            .into(),
            ErrorKind::ExpectedRequest => {
                Error::new(typ, "Expected request message".to_owned().into())
                    .into()
            }
            ErrorKind::ExpectedControl => {
                Error::new(typ, "Expected control message".to_owned().into())
                    .into()
            }
            ErrorKind::ExpectedInfo => {
                Error::new(typ, "Expected info message".to_owned().into())
                    .into()
            }
            ErrorKind::ExpectedRequestOrControl => Error::new(
                typ,
                "Expected request or control message".to_owned().into(),
            )
            .into(),
            ErrorKind::InternalError => Error::new(
                typ,
                "Error occured when trying to fulfill request."
                    .to_owned()
                    .into(),
            )
            .into(),
        }
    }
}

impl From<Error> for MsgMessage {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

impl From<PlayMsg> for MsgMessage {
    fn from(value: PlayMsg) -> Self {
        Self::Play(value)
    }
}

impl From<Request> for MsgMessage {
    fn from(value: Request) -> Self {
        Self::Request(value)
    }
}

impl From<AnyControlMsg> for MsgMessage {
    fn from(value: AnyControlMsg) -> Self {
        Self::Control(value)
    }
}

impl From<ControlMsg> for MsgMessage {
    fn from(value: ControlMsg) -> Self {
        Self::Control(value.into())
    }
}

impl From<DataControlMsg> for MsgMessage {
    fn from(value: DataControlMsg) -> Self {
        Self::Control(value.into())
    }
}

impl From<DataResponse> for MsgMessage {
    fn from(value: DataResponse) -> Self {
        Self::Data(value)
    }
}
