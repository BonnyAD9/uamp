use eyre::Result;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use std::{
    io::{BufReader, BufWriter, Write},
    net::TcpStream,
};

use crate::uamp_app::{UampApp, UampMessage};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Error(Error),
    Request(Request),
    Control(Control),
    Info(Info),
    Success,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    typ: ErrorType,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorType {
    DeserializeFailed,
    ExpectedRequest,
    ExpectedControl,
    ExpectedInfo,
    ExpectedRequestOrControl,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {}

#[derive(Debug, Serialize, Deserialize)]
pub enum Control {
    PlayPause,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {}

pub struct Messenger<'a> {
    reader: BufReader<&'a TcpStream>,
    writer: BufWriter<&'a TcpStream>,
}

impl<'a> Messenger<'a> {
    pub fn try_new(stream: &'a TcpStream) -> Result<Self> {
        Ok(Self {
            reader: BufReader::new(stream),
            writer: BufWriter::new(stream),
        })
    }

    pub fn send(&mut self, msg: Message) -> Result<()> {
        let mut ser = Serializer::new(&mut self.writer);
        msg.serialize(&mut ser)?;
        _ = self.writer.flush();
        Ok(())
    }

    pub fn recieve(&mut self) -> Result<Message> {
        Ok(rmp_serde::from_read(&mut self.reader)?)
    }
}

macro_rules! extract {
    ($fn_name:ident, $type_name:ident) => {
        pub fn $fn_name(self) -> Option<$type_name> {
            if let Self::$type_name(e) = self {
                Some(e)
            } else {
                None
            }
        }
    };
}

impl Error {
    pub fn new(typ: ErrorType, message: String) -> Self {
        Error { typ, message }
    }
}

impl Message {
    extract!(error, Error);
    extract!(request, Request);
    extract!(control, Control);
    extract!(info, Info);

    pub fn is_error(&self) -> bool {
        matches!(self, Message::Error(_))
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Message::Success)
    }

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

pub fn error(typ: ErrorType, message: String) -> Message {
    Message::Error(Error::new(typ, message))
}

impl UampApp {
    pub fn message_event(msg: Message) -> (Message, Option<UampMessage>) {
        let msg = if let Some(msg) = msg.control() {
            msg
        } else {
            return (Message::new_error(ErrorType::ExpectedControl), None);
        };

        (Message::Success, Some(msg.into()))
    }
}
