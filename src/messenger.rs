use log::warn;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use std::{
    io::{BufReader, BufWriter, Write},
    net::TcpStream,
};

use crate::{
    err::Result,
    uamp_app::{ControlMsg, UampApp, UampMessage},
};

/// Messages passed between uamp instances
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    /// An error occured
    Error(Error),
    /// todo
    Request(Request),
    /// Send simple action to be done
    Control(ControlMsg),
    /// todo
    Info(Info),
    /// Message indicating success
    Success,
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
}

/// todo
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {}

/// todo
#[derive(Debug, Serialize, Deserialize)]
pub struct Info {}

/// used to send messages across instances
pub struct Messenger<'a> {
    reader: BufReader<&'a TcpStream>,
    writer: BufWriter<&'a TcpStream>,
}

impl<'a> Messenger<'a> {
    /// Tries to create a new [`Messenger`] from a tcp stream.
    ///
    /// # Errors
    /// - cannot fail as of now
    pub fn try_new(stream: &'a TcpStream) -> Result<Self> {
        Ok(Self {
            reader: BufReader::new(stream),
            writer: BufWriter::new(stream),
        })
    }

    /// Try to send a message.
    ///
    /// # Errors
    /// - Failed to serialize
    pub fn send(&mut self, msg: Message) -> Result<()> {
        let mut ser = Serializer::new(&mut self.writer);
        msg.serialize(&mut ser)?;
        if let Err(e) = self.writer.flush() {
            warn!("Failed to flush message: {}", e);
        }
        Ok(())
    }

    /// Recieve a message
    ///
    /// # Errors
    /// - Failed to deserialize
    pub fn recieve(&mut self) -> Result<Message> {
        Ok(rmp_serde::from_read(&mut self.reader)?)
    }
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

impl Error {
    /// Creates a new error with the given type and message
    pub fn new(typ: ErrorType, message: String) -> Self {
        Error { typ, message }
    }
}

impl Message {
    extract!(error, Error, Error);
    extract!(request, Request, Request);
    extract!(control, ControlMsg, Control);
    extract!(info, Info, Info);

    /// Returns true if the message is error message
    pub fn is_error(&self) -> bool {
        matches!(self, Message::Error(_))
    }

    /// Returns true if the message is success message
    pub fn is_success(&self) -> bool {
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
        }
    }
}

/// Creates new error message from its type and message
pub fn error(typ: ErrorType, message: String) -> Message {
    Message::Error(Error::new(typ, message))
}

impl UampApp {
    /// Translates [`Message`] to [`UampMessage`].
    ///
    /// Returns message that should be sent as a response and the translated
    /// [`UampMessage`] if it should produce one.
    pub fn message_event(msg: Message) -> (Message, Option<UampMessage>) {
        let msg = if let Some(msg) = msg.control() {
            msg
        } else {
            return (Message::new_error(ErrorType::ExpectedControl), None);
        };

        (Message::Success, Some(UampMessage::Control(msg)))
    }
}
