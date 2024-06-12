
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
        command::AppCtrl, err::Result, messenger::{Error, ErrorKind}, msg::{FnDelegate, Msg}
    },
};

use super::{Info, MsgMessage, Request};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Used to send messages across instances.
#[derive(Debug)]
pub struct Messenger<'a> {
    reader: BufReader<&'a TcpStream>,
    writer: BufWriter<&'a TcpStream>,
}

impl<'a> Messenger<'a> {
    /// Creates a new [`Messenger`] from a tcp stream.
    pub fn new(stream: &'a TcpStream) -> Self {
        Self {
            reader: BufReader::new(stream),
            writer: BufWriter::new(stream),
        }
    }

    /// Try to send a message.
    ///
    /// # Errors
    /// - Failed to serialize
    pub fn send(&mut self, msg: MsgMessage) -> Result<()> {
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
    pub fn recieve(&mut self) -> Result<MsgMessage> {
        Ok(rmp_serde::from_read(&mut self.reader)?)
    }
}

impl UampApp {
    /// Translates [`Message`] to [`UampMessage`].
    ///
    /// Returns message that should be sent as a response and the translated
    /// [`UampMessage`] if it should produce one.
    pub fn message_event(
        msg: MsgMessage,
        stream: &TcpStream,
    ) -> (Option<MsgMessage>, Option<Msg>) {
        match msg {
            MsgMessage::Request(Request::Info) => {
                let stream = match stream.try_clone() {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Failed to clone tcp stream: {e}");
                        return (
                            Some(MsgMessage::Error(Error::new(
                                ErrorKind::InternalError,
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
                        move |app: &mut UampApp, _: &mut AppCtrl| {
                            let mut msg = Messenger::new(&stream);
                            if let Err(e) =
                                msg.send(MsgMessage::Info(Box::new(Info {
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
                                })))
                            {
                                error!("Failed to send message: {e}");
                            };
                            None
                        },
                    )),
                )
            }
            MsgMessage::Control(msg) => {
                (Some(MsgMessage::Success), Some(msg.into()))
            }
            MsgMessage::Play(msg) => {
                (Some(MsgMessage::Success), Some(Msg::PlaySong(msg)))
            }
            _ => (Some(MsgMessage::new_error(ErrorKind::ExpectedControl)), None),
        }
    }
}
