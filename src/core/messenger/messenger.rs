use log::{error, warn};
use rmp_serde::Serializer;
use serde::Serialize;
use std::{
    io::{BufReader, BufWriter, Write},
    net::TcpStream,
};

use crate::{
    app::UampApp,
    core::{
        err::Result,
        msg::{FnDelegate, Msg},
    },
};

use super::msg::{Error, ErrorType, Info, Message, Request};

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
    pub fn message_event(
        msg: Message,
        stream: &TcpStream,
    ) -> (Option<Message>, Option<Msg>) {
        match msg {
            Message::Request(Request::Info) => {
                let stream = match stream.try_clone() {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Failed to clone tcp stream: {e}");
                        return (
                            Some(Message::Error(Error::new(
                                ErrorType::InternalError,
                                format!(
                                "Error occured when trying to fulfill request\
                                : {e}"
                            ),
                            ))),
                            None,
                        );
                    }
                };
                (
                    None,
                    Some(Msg::delegate::<_, FnDelegate<_>>(
                        move |app: &mut UampApp| {
                            let mut msg = match Messenger::try_new(&stream) {
                                Ok(m) => m,
                                Err(e) => {
                                    error!("Failed to create messenger: {e}");
                                    return None;
                                }
                            };
                            if let Err(e) = msg.send(Message::Info(Info {
                                version: option_env!("CARGO_PKG_VERSION")
                                    .unwrap_or("unknown")
                                    .to_owned(),
                                now_playing: app
                                    .player
                                    .now_playing()
                                    .map(|i| app.library[i].clone()),
                                playlist_len: app.player.playlist().len(),
                                playlist_pos: app.player.current(),
                                is_playing: app.player.is_playing(),
                                timestamp: app.player.timestamp(),
                            })) {
                                error!("Failed to send message: {e}");
                            };
                            None
                        },
                    )),
                )
            }
            Message::Control(msg) => {
                (Some(Message::Success), Some(Msg::Control(msg)))
            }
            _ => (Some(Message::new_error(ErrorType::ExpectedControl)), None),
        }
    }
}
