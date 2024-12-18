use std::{
    mem,
    net::TcpStream,
    path::Path,
    time::{Duration, Instant},
};

use log::error;
use pareg::{has_any_key, Pareg};
use termal::eprintacln;

use crate::core::{
    config::Config,
    messenger::{DataResponse, Info, Messenger, MsgMessage, Request},
    Error, PlayMsg, Result,
};

use super::{help::help_instance, port::Port, printer};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Messages to be sent to a running instance.
#[derive(Default, Debug)]
pub struct Instance {
    /// Messages to send to a running instance.
    pub messages: Vec<MsgMessage>,
    /// Port number of the server of the running instance.
    pub port: Option<u16>,
    /// Server address of the running instance.
    pub server: Option<String>,
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
                "info" | "nfo" => {
                    self.messages.push(Request::Info(1, 3).into())
                }
                v if has_any_key!(v, '=', "p", "play") => {
                    self.messages.push(
                        PlayMsg::TmpPath(
                            args.cur_val::<&Path>('=')?
                                .canonicalize()
                                .map_err(|e| {
                                    Error::io(e).msg(format!(
                                        "Cannot find the file `{}`",
                                        args.cur().unwrap()
                                    ))
                                })?
                                .into(),
                        )
                        .into(),
                    );
                }
                v if has_any_key!(v, '=', "query", "list", "l") => {
                    self.messages.push(
                        Request::Query(
                            args.cur_mval('=')?.unwrap_or_default(),
                        )
                        .into(),
                    );
                }
                "-h" | "-?" | "--help" => help_instance(color),
                "-p" | "--port" => {
                    self.port = Some(args.next_arg::<Port>()?.0)
                }
                "-a" | "--address" => self.server = Some(args.next_arg()?),
                "--" => break,
                _ => self.messages.push(MsgMessage::Control(args.cur_arg()?)),
            }
        }

        Ok(())
    }

    /// Sends the messages to the running instance.
    ///
    /// # Errors
    /// - There is no running instance of uamp with the server address and
    ///   port.
    pub fn send(mut self, conf: &Config, color: bool) -> Result<()> {
        self.port = self.port.or(Some(conf.port()));
        self.server = self
            .server
            .take()
            .or_else(|| Some(conf.server_address().to_owned()));

        for m in mem::take(&mut self.messages) {
            let send_time = Instant::now();
            let res = self.send_message(m);
            match res {
                Ok(MsgMessage::Success) => {}
                Ok(MsgMessage::Data(i)) => {
                    Self::print_data(i, color, send_time);
                }
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

    fn print_data(data: DataResponse, color: bool, send_time: Instant) {
        match data {
            DataResponse::Info(i) => Self::print_info(*i, color),
            DataResponse::SongList(songs) => {
                printer::song_list(songs, color, send_time)
            }
        }
    }

    fn print_info(info: Info, color: bool) {
        printer::now_playing(&info, color);

        if !info.before.is_empty() || !info.after.is_empty() {
            printer::playlist(&info, color);
        }

        printer::playlist_config(&info, color);
        printer::footer(&info, color);
    }
}
