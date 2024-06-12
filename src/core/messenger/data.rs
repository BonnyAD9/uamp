use serde::{Deserialize, Serialize};

use raplay::Timestamp;

use crate::{
    core::msg::{AnyControlMsg, PlayMsg},
    library::Song,
};

use super::MsgMessage;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Describes error over the internet.
#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    /// Describes the kind of the error.
    kind: ErrorKind,
    /// Message suitable for the user.
    message: String,
}

/// Describes the kind of the error
#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorKind {
    /// Failed to deserialize the sent mesage.
    DeserializeFailed,
    /// Expected that a request message was sent but there was a different
    /// message.
    ExpectedRequest,
    /// Expected that a control message was sent but there was a different
    /// message.
    ExpectedControl,
    /// Expected that a info message was sent but there was a different
    /// message.
    ExpectedInfo,
    /// Expected that a request or control message was sent but there was a
    /// different message.
    ExpectedRequestOrControl,
    /// Error occured when trying to do what was requested.
    InternalError,
}


/// Request someting from the other side.
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// Request the current playback info.
    Info,
}

/// Info about the playback.
#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    /// Uamp version.
    pub version: String,
    /// Song that is now playing.
    pub now_playing: Option<Song>,
    /// Length of the playlisg.
    pub playlist_len: usize,
    /// Current position in the playlist.
    pub playlist_pos: Option<usize>,
    /// True if is currently playing.
    pub is_playing: bool,
    /// The timestamp of the current playback.
    pub timestamp: Option<Timestamp>,
}

impl Error {
    /// Creates a new error with the given type and message.
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Error { kind, message }
    }
}
