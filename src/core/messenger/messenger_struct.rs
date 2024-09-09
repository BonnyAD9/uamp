use log::{error, warn};
use rmp_serde::Serializer;
use serde::Serialize;
use std::{
    io::{BufReader, BufWriter, Write},
    net::TcpStream,
};

use crate::{
    core::{
        messenger::{DataResponse, Error, ErrorKind},
        query::Query,
        FnDelegate, Msg, Result, UampApp,
    },
    env::AppCtrl,
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
    pub(in crate::core) fn message_event(
        msg: MsgMessage,
        stream: &TcpStream,
    ) -> (Option<MsgMessage>, Option<Msg>) {
        match msg {
            MsgMessage::Request(r) => Self::handle_request(r, stream),
            MsgMessage::Control(msg) => {
                (Some(MsgMessage::Success), Some(msg.into()))
            }
            MsgMessage::Play(msg) => {
                (Some(MsgMessage::Success), Some(Msg::PlaySong(msg)))
            }
            _ => (
                Some(MsgMessage::new_error(ErrorKind::ExpectedControl)),
                None,
            ),
        }
    }

    fn handle_request(
        req: Request,
        stream: &TcpStream,
    ) -> (Option<MsgMessage>, Option<Msg>) {
        match req {
            Request::Info => Self::request_info(stream),
            Request::Query(f) => Self::query_request(stream, f),
        }
    }

    fn request_info(stream: &TcpStream) -> (Option<MsgMessage>, Option<Msg>) {
        let stream = match Self::clone_stream(stream) {
            Ok(s) => s,
            Err(e) => return e,
        };

        let delegate = Msg::delegate::<_, FnDelegate<_>>(
            move |app: &mut UampApp, _: &mut AppCtrl| {
                app.info_response(&stream)
            },
        );

        (None, Some(delegate))
    }

    fn info_response(&mut self, stream: &TcpStream) -> Vec<Msg> {
        let mut msg = Messenger::new(stream);

        let info = Info {
            version: option_env!("CARGO_PKG_VERSION")
                .unwrap_or("unknown")
                .to_owned(),
            now_playing: self
                .player
                .now_playing()
                .map(|i| self.library[i].clone()),
            playlist_len: self.player.playlist().len(),
            playlist_pos: self.player.playlist().get_pos(),
            is_playing: self.player.is_playing(),
            timestamp: self.player.timestamp(),
        };

        if let Err(e) = msg.send(DataResponse::Info(Box::new(info)).into()) {
            error!("Failed to send message: {e}");
        };

        vec![]
    }

    fn query_request(
        stream: &TcpStream,
        query: Query,
    ) -> (Option<MsgMessage>, Option<Msg>) {
        let stream = match Self::clone_stream(stream) {
            Ok(s) => s,
            Err(e) => return e,
        };

        let delegate = Msg::delegate::<_, FnDelegate<_>>(
            move |app: &mut UampApp, _: &mut AppCtrl| {
                app.query_response(&stream, &query)
            },
        );

        (None, Some(delegate))
    }

    fn query_response(
        &mut self,
        stream: &TcpStream,
        query: &Query,
    ) -> Vec<Msg> {
        let mut msg = Messenger::new(stream);

        let songs = query.clone_songs(
            &self.library,
            self.config.simple_sorting(),
            self.library.iter(),
        );

        if let Err(e) = msg.send(DataResponse::SongList(songs).into()) {
            error!("Failed to send message: {e}");
        }

        vec![]
    }

    fn clone_stream(
        stream: &TcpStream,
    ) -> std::result::Result<TcpStream, (Option<MsgMessage>, Option<Msg>)>
    {
        stream.try_clone().map_err(|e| {
            error!("Failed to clone tcp stream: {e}");
            (
                Some(
                    Error::new(
                        ErrorKind::InternalError,
                        format!(
                            "Error occured when trying to fulfill request\
                    : {e}"
                        ),
                    )
                    .into(),
                ),
                None,
            )
        })
    }
}
