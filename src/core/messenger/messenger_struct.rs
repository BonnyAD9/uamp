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
        msg.serialize(&mut ser).map_err(|e| {
            crate::core::Error::SerdeRmpEncode(e.into())
                .msg("Failed to encode message for TCP.")
        })?;
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
        rmp_serde::from_read(&mut self.reader).map_err(|e| {
            crate::core::Error::SerdeRmpDecode(e.into())
                .msg("Failed to decode message from TCP.")
        })
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
            MsgMessage::Control(msg) => Self::handle_msg(msg.into(), stream),
            MsgMessage::Play(msg) => Self::handle_msg(msg.into(), stream),
            _ => (
                Some(MsgMessage::new_error(ErrorKind::ExpectedControl)),
                None,
            ),
        }
    }

    fn handle_msg(
        msg: Msg,
        stream: &TcpStream,
    ) -> (Option<MsgMessage>, Option<Msg>) {
        let stream = match Self::clone_stream(stream) {
            Ok(s) => s,
            // Fallback to simple control message without feedback.
            Err(_) => return (Some(MsgMessage::Success), Some(msg)),
        };

        let delegate = Msg::delegate::<_, FnDelegate<_>>(
            move |app: &mut UampApp, ctrl: &mut AppCtrl| {
                Self::control_response(
                    app.update_err(ctrl, msg.clone()),
                    &stream,
                )
                .map(|_| vec![])
            },
        );

        (None, Some(delegate))
    }

    fn control_response(r: Result<()>, stream: &TcpStream) -> Result<()> {
        let mut msg = Messenger::new(stream);

        match r {
            Err(e) => {
                if let Err(e2) = msg.send(MsgMessage::Error(Error::new(
                    ErrorKind::InternalError,
                    e.clone_universal(),
                ))) {
                    crate::core::Error::multiple([e, e2].into())
                } else {
                    Err(e)
                }
            }
            Ok(_) => msg.send(MsgMessage::Success),
        }
    }

    fn handle_request(
        req: Request,
        stream: &TcpStream,
    ) -> (Option<MsgMessage>, Option<Msg>) {
        match req {
            Request::Info(before, after) => {
                Self::request_info(stream, before, after)
            }
            Request::Query(f) => Self::query_request(stream, f),
        }
    }

    fn request_info(
        stream: &TcpStream,
        before: usize,
        after: usize,
    ) -> (Option<MsgMessage>, Option<Msg>) {
        let stream = match Self::clone_stream(stream) {
            Ok(s) => s,
            Err(e) => return e,
        };

        let delegate = Msg::delegate::<_, FnDelegate<_>>(
            move |app: &mut UampApp, _: &mut AppCtrl| {
                app.info_response(&stream, before, after)
            },
        );

        (None, Some(delegate))
    }

    fn info_response(
        &mut self,
        stream: &TcpStream,
        before: usize,
        after: usize,
    ) -> Result<Vec<Msg>> {
        let mut msg = Messenger::new(stream);

        let idx = self.player.playlist().current_idx();
        let (before, after) = if let Some(idx) = idx {
            let start = idx.saturating_sub(before);
            let end = (idx + after + 1).min(self.player.playlist().len());
            (
                self.player.playlist()[start..idx]
                    .iter()
                    .map(|i| self.library[*i].clone())
                    .collect(),
                self.player.playlist()[idx + 1..end]
                    .iter()
                    .map(|i| self.library[*i].clone())
                    .collect(),
            )
        } else {
            (vec![], vec![])
        };

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
            volume: self.player.volume(),
            mute: self.player.mute(),
            timestamp: self.player.timestamp(),
            before,
            after,
            playlist_stack: self
                .player
                .playlist_stack()
                .iter()
                .map(|p| (p.current_idx(), p.len()))
                .collect(),
            playlist_end: self
                .player
                .playlist()
                .on_end
                .as_ref()
                .or(self.config.default_playlist_end_action().as_ref())
                .cloned(),
            playlist_add_policy: self.player.playlist().add_policy,
        };

        msg.send(DataResponse::Info(Box::new(info)).into())
            .map_err(|e| e.prepend("Failed to send message."))
            .map(|_| vec![])
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
    ) -> Result<Vec<Msg>> {
        let mut msg = Messenger::new(stream);

        let songs = query.clone_songs(
            &self.library,
            self.config.simple_sorting(),
            self.library.iter(),
        );

        msg.send(DataResponse::SongList(songs).into())
            .map_err(|e| e.prepend("Failed to send message."))
            .map(|_| vec![])
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
                            "Error occured when trying to fulfill request: {e}"
                        )
                        .into(),
                    )
                    .into(),
                ),
                None,
            )
        })
    }
}
