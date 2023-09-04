use log::warn;
use rmp_serde::Serializer;
use serde::Serialize;
use std::{
    io::{BufReader, BufWriter, Write},
    net::TcpStream,
};

use crate::{core::{err::Result, msg::Msg}, app::UampApp};

use super::msg::{Message, ErrorType};

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

impl UampApp {
    /// Translates [`Message`] to [`UampMessage`].
    ///
    /// Returns message that should be sent as a response and the translated
    /// [`UampMessage`] if it should produce one.
    pub fn message_event(msg: Message) -> (Message, Option<Msg>) {
        let msg = if let Some(msg) = msg.control() {
            msg
        } else {
            return (Message::new_error(ErrorType::ExpectedControl), None);
        };

        (Message::Success, Some(Msg::Control(msg)))
    }
}
