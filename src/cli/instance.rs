use std::{borrow::Cow, mem, net::TcpStream, path::Path, time::Duration};

use log::error;
use pareg::{ArgIterator, ByRef};
use termal::printcln;

use crate::{
    config::Config,
    core::{
        extensions::duration_to_string,
        messenger::{
            Info, Messenger, MsgMessage, Request
        },
        msg::PlayMsg,
        Result,
    },
    starts,
};

use super::help::help_instance;

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
    pub(super) fn parse<'a, I>(
        &mut self,
        args: &mut ArgIterator<'a, I>,
    ) -> Result<()>
    where
        I: Iterator,
        I::Item: ByRef<&'a str>,
    {
        while let Some(arg) = args.next() {
            match arg {
                "info" | "nfo" => {
                    self.messages.push(MsgMessage::Request(Request::Info))
                }
                v if starts!(v, "p" | "play") => {
                    self.messages.push(MsgMessage::Play(PlayMsg::TmpPath(
                        args.cur_key_val::<&str, &Path>('=')?
                            .1
                            .canonicalize()?
                            .into(),
                    )));
                }
                "-h" | "-?" | "--help" => help_instance(),
                "-p" | "--port" => self.port = Some(args.next_arg()?),
                "-a" | "--address" => self.server = Some(args.next_arg()?),
                "--" => break,
                _ => {
                    self.messages.push(MsgMessage::Control(args.cur_arg()?))
                }
            }
        }

        Ok(())
    }

    /// Sends the messages to the running instance.
    ///
    /// # Errors
    /// - There is no running instance of uamp with the server address and
    ///   port.
    pub fn send(mut self, conf: &Config) -> Result<()> {
        self.port = self.port.or(Some(conf.port()));
        self.server = self
            .server
            .take()
            .or_else(|| Some(conf.server_address().to_owned()));

        for m in mem::take(&mut self.messages) {
            let res = self.send_message(m);
            match res {
                Ok(MsgMessage::Success) => {}
                Ok(MsgMessage::Info(i)) => {
                    Self::print_info(*i);
                }
                Err(e) => println!("{e}"),
                Ok(r) => {
                    println!("Unexpected response: {r:?}");
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
        ))?;
        if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(5))) {
            error!("failed to send message: {}", e);
        }

        let mut msgr = Messenger::new(&stream);

        msgr.send(msg)?;

        msgr.recieve()
    }

    fn print_info(info: Info) {
        let mut title: Cow<str> = "--".into();
        let mut artist: Cow<str> = "--".into();
        let mut album: Cow<str> = "--".into();
        let state = if info.is_playing { "||" } else { "|>" };
        let mut cur: Cow<str> = "--:--".into();
        let mut pos: Cow<str> = "?".into();
        let mut total: Cow<str> = "--:--".into();
        let plen = info.playlist_len.to_string();
        let mut disc: Cow<str> = "0".into();
        let mut track: Cow<str> = "0".into();
        let version = format!("v{}", info.version);
        let mut before: Cow<str> = "".into();
        let mut is = "";
        let mut after: Cow<str> = format!("{:->70}", '-').into();

        if let Some(s) = &info.now_playing {
            title = s.title().into();
            artist = s.artist().into();
            album = s.album().into();
            disc = s.disc_str().into();
            track = s.track_str().into();
        }

        if let Some(p) = info.playlist_pos {
            pos = p.to_string().into();
        }

        if let Some(t) = info.timestamp {
            cur = duration_to_string(t.current, true).into();
            total = duration_to_string(t.total, true).into();
            let n = (t.current.as_secs_f32() / t.total.as_secs_f32() * 70.)
                as usize;
            before = format!("{:=>n$}", '=').into();
            is = "#";
            let m = 69 - n;
            after = format!("{:->m$}", '-').into();
            match n {
                0 => before = "".into(),
                70 => {
                    after = "".into();
                    is = "";
                }
                69 => after = "".into(),
                _ => {}
            }
        }

        let blen = (80 - artist.len() - album.len() - 9) / 2;
        let playlist = format!("{pos}/{plen}");
        let dt = format!("<{disc}-{track}>");

        printcln!(
            "
{'bold y}{title: ^80}{'_}
{: >blen$}{'gr}by {'dc}{artist} {'gr}from {'dm}{album}{'_}

     {'w}{cur: <27}{'_ bold}<||    {'y}{state}    {'_fg}||>{'_ w}{total: >27}{'_}
    {'bold}[{'_ y}{before}{'w bold}{is}{'_ gr}{after}{'_ bold}]{'_}

{'gr}{playlist: ^80}
{dt: ^80}
uamp{version: >76}{'_}",
            ' '
        );
    }
}
