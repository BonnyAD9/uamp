use serde::{Deserialize, Serialize};

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

impl Error {
    /// Creates a new error with the given type and message.
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Error { kind, message }
    }
}
