use std::{
    mem,
    net::TcpStream,
    time::{Duration, Instant},
};

use log::error;
use pareg::{Pareg, has_any_key, parse_arg};
use termal::eprintacln;

use crate::core::{
    AnyControlMsg, DataControlMsg, Error, Result,
    config::Config,
    messenger::{DataResponse, Messenger, MsgMessage, Request},
};

use super::{
    Props, help::help_instance, playlist_range::PlaylistRange, port::Port,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Messages to be sent to a running instance.
#[derive(Default, Debug)]
pub struct Instance {
    /// Messages to send to a running instance.
    pub messages: Vec<(MsgMessage, Intention)>,
    /// Port number of the server of the running instance.
    pub port: Option<u16>,
    /// Server address of the running instance.
    pub server: Option<String>,
    /// Verbosity override for this actoin
    verbosity: Option<i32>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub enum Intention {
    #[default]
    Default,
    Clear,
}

impl Instance {
    /// Parses the instance action arguments.
    ///
    /// # Errors
    /// - The arguments are invalid.
    pub(super) fn parse(
        &mut self,
        args: &mut Pareg,
        color: bool,
    ) -> Result<()> {
        while let Some(arg) = args.next() {
            match arg {
                v if has_any_key!(v, '=', "info", "nfo") => {
                    let s = args
                        .cur_mval::<PlaylistRange>('=')?
                        .unwrap_or(PlaylistRange(1, 3));
                    self.messages.push((
                        Request::Info(s.0, s.1).into(),
                        Intention::Default,
                    ))
                }
                v if has_any_key!(v, '=', "show") => {
                    let s = args
                        .cur_mval::<PlaylistRange>('=')?
                        .unwrap_or(PlaylistRange(1, 3));
                    self.messages.push((
                        Request::Info(s.0, s.1).into(),
                        Intention::Clear,
                    ))
                }
                v if has_any_key!(v, '=', "query", "list", "l") => {
                    self.messages.push((
                        Request::Query(
                            args.cur_mval('=')?.unwrap_or_default(),
                        )
                        .into(),
                        Intention::Default,
                    ));
                }
                "-h" | "-?" | "--help" => help_instance(color),
                "-p" | "--port" => {
                    self.port = Some(args.next_arg::<Port>()?.0)
                }
                "-a" | "--address" => self.server = Some(args.next_arg()?),
                "-v" | "--verbose" => self.verbosity = Some(1),
                v if v.starts_with("-v") => {
                    self.verbosity =
                        Some(args.cur_manual(|a| parse_arg(&a[2..]))?);
                }
                "--" => break,
                _ => self.messages.push((
                    MsgMessage::Control(args.cur_arg()?),
                    Intention::Default,
                )),
            }
        }

        Ok(())
    }

    /// Sends the messages to the running instance.
    ///
    /// # Errors
    /// - There is no running instance of uamp with the server address and
    ///   port.
    pub fn send(mut self, conf: &Config, props: &Props) -> Result<()> {
        self.port = self.port.or(Some(conf.port()));
        self.server = self
            .server
            .take()
            .or_else(|| Some(conf.server_address().to_owned()));

        for (m, i) in mem::take(&mut self.messages) {
            let send_time = Instant::now();
            let restarting = matches!(
                m,
                MsgMessage::Control(AnyControlMsg::Data(ref d)) if
                matches!(&**d, DataControlMsg::Restart(_))
            );
            let res = self.send_message(m);
            match res {
                Ok(MsgMessage::Success) => {}
                Ok(MsgMessage::Data(d)) => {
                    self.print_data(d, conf, props, send_time, i);
                }
                // When restarting, the restarted uamp will not answer and this
                // error will occur. This is expected so we don't want to alert
                // the user.
                Err(Error::SerdeRmpDecode(e))
                    if restarting
                        && matches!(
                            e.inner(),
                            rmp_serde::decode::Error::InvalidMarkerRead(e)
                                if e.kind()
                                    == std::io::ErrorKind::UnexpectedEof
                        ) => {}
                Err(e) => eprintacln!("{e}"),
                Ok(MsgMessage::Error(e)) => {
                    eprintln!("{}", e.ctx);
                }
                Ok(r) => {
                    eprintacln!("{'r}error: {'_}Unexpected response: {r:?}");
                }
            }
        }

        Ok(())
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl Instance {
    fn send_message(&self, msg: MsgMessage) -> Result<MsgMessage> {
        assert!(self.server.is_some());
        assert!(self.port.is_some());

        let stream = TcpStream::connect(format!(
            "{}:{}",
            self.server.as_ref().unwrap(),
            self.port.unwrap(),
        ))
        .map_err(|e| {
            Error::io(e)
                .msg("Failed to connect to uamp.")
                .hint("Is uamp server running?")
        })?;
        if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(5))) {
            eprintln!("Failed to set TCP timeout: {e}");
            error!("Failed to set TCP timeout: {e}");
        }

        let mut msgr = Messenger::new(&stream);

        msgr.send(msg)?;

        msgr.recieve()
    }

    fn print_data(
        &self,
        data: DataResponse,
        conf: &Config,
        props: &Props,
        send_time: Instant,
        intention: Intention,
    ) {
        match data {
            DataResponse::Info(i) => props.print_style.info(
                &i,
                conf,
                props.color,
                intention == Intention::Clear,
            ),
            DataResponse::SongList(songs) => props.print_style.song_list(
                &songs,
                &props.with_verbosity(self.verbosity),
                send_time,
            ),
        }
    }
}
