use std::time::Duration;

use serde::{Deserialize, Serialize};

use raplay::Timestamp;

use crate::{
    core::msg::{ControlMsg, PlayMsg},
    library::Song,
};

/// Messages passed between uamp instances
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    /// An error occured
    Error(Error),
    /// Play something
    Play(PlayMsg),
    /// todo
    Request(Request),
    /// Send simple action to be done
    Control(ControlMsg),
    /// todo
    Info(Box<Info>),
    /// Message indicating success
    Success,
    /// Wait for the given time and exit
    WaitExit(Duration),
    Ping,
}

/// Describes error over the internet
#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    /// Describes the kind of the error
    typ: ErrorType,
    /// Message suitable for the user
    message: String,
}

/// Describes the kind of the error
#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorType {
    /// Failed to deserialize the sent mesage
    DeserializeFailed,
    /// Expected that a request message was sent but there was a different
    /// message
    ExpectedRequest,
    /// Expected that a control message was sent but there was a different
    /// message
    ExpectedControl,
    /// Expected that a info message was sent but there was a different
    /// message
    ExpectedInfo,
    /// Expected that a request or control message was sent but there was a
    /// different message
    ExpectedRequestOrControl,
    /// Error occured when trying to do what was requested
    InternalError,
}

/// todo
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// Request the current playback info
    Info,
}

/// Info about the playback
#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    pub version: String,
    /// Song that is now playing
    pub now_playing: Option<Song>,
    /// Length of the playlisg
    pub playlist_len: usize,
    /// Current position in the playlist
    pub playlist_pos: Option<usize>,
    /// True if is currently playing
    pub is_playing: bool,
    /// The timestamp of the current playback
    pub timestamp: Option<Timestamp>,
}

/// Creates extracton method for the given message variant
///
/// # Example
/// ```
/// extract!(fn_name, TypeName, EnumVariant);
/// ```
macro_rules! extract {
    ($fn_name:ident, $type_name:ident, $variant:ident) => {
        pub fn $fn_name(self) -> Option<$type_name> {
            if let Self::$variant(e) = self {
                Some(e)
            } else {
                None
            }
        }
    };
}

/// Messanger message
impl Message {
    extract!(_error, Error, Error);
    extract!(_request, Request, Request);
    extract!(_control, ControlMsg, Control);

    pub fn _info(self) -> Option<Info> {
        if let Self::Info(e) = self {
            Some(*e)
        } else {
            None
        }
    }

    /// Returns true if the message is error message
    pub fn _is_error(&self) -> bool {
        matches!(self, Message::Error(_))
    }

    /// Returns true if the message is success message
    pub fn _is_success(&self) -> bool {
        matches!(self, Message::Success)
    }

    /// Creates new error with a default message for its type
    pub fn new_error(typ: ErrorType) -> Self {
        match typ {
            ErrorType::DeserializeFailed => Self::Error(Error::new(
                typ,
                "Failed to deserialize the incoming message".to_owned(),
            )),
            ErrorType::ExpectedRequest => Self::Error(Error::new(
                typ,
                "Expected request message".to_owned(),
            )),
            ErrorType::ExpectedControl => Self::Error(Error::new(
                typ,
                "Expected control message".to_owned(),
            )),
            ErrorType::ExpectedInfo => Self::Error(Error::new(
                typ,
                "Expected info message".to_owned(),
            )),
            ErrorType::ExpectedRequestOrControl => Self::Error(Error::new(
                typ,
                "Expected request or control message".to_owned(),
            )),
            ErrorType::InternalError => Self::Error(Error::new(
                typ,
                "Error occured when trying to fulfill request.".to_owned(),
            )),
        }
    }
}

impl Error {
    /// Creates a new error with the given type and message
    pub fn new(typ: ErrorType, message: String) -> Self {
        Error { typ, message }
    }
}
