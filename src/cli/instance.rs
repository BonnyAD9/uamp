use std::{mem, net::TcpStream, time::Duration};

use log::error;
use pareg::{ArgIterator, ByRef};

use crate::{
    config::Config,
    core::{
        extensions::duration_to_string,
        messenger::{
            msg::{self, Info, Request},
            Messenger,
        },
        Result,
    },
};

use super::help::help_instance;

#[derive(Default)]
pub struct Instance {
    pub messages: Vec<msg::Message>,
    pub port: Option<u16>,
    pub server: Option<String>,
}

impl Instance {
    /// Parses the instance action arguments
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
                "info" => {
                    self.messages.push(msg::Message::Request(Request::Info))
                }
                "-h" | "-?" | "--help" => help_instance(),
                "-p" | "--port" => self.port = Some(args.next_arg()?),
                "-a" | "--address" => self.server = Some(args.next_arg()?),
                "--" => break,
                _ => {
                    self.messages.push(msg::Message::Control(args.cur_arg()?))
                }
            }
        }

        Ok(())
    }

    pub fn send(mut self, conf: &Config) -> Result<()> {
        self.port = self.port.or(Some(conf.port()));
        self.server = self
            .server
            .take()
            .or_else(|| Some(conf.server_address().to_owned()));

        for m in mem::take(&mut self.messages) {
            let res = self.send_message(m);
            match res {
                Ok(msg::Message::Success) => {}
                Ok(msg::Message::Info(i)) => {
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

    /// Sends message to a existing uamp instance
    fn send_message(&self, msg: msg::Message) -> Result<msg::Message> {
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

        let mut msgr = Messenger::try_new(&stream)?;

        msgr.send(msg)?;

        msgr.recieve()
    }

    /// Prints the info about instance playback
    fn print_info(info: Info) {
        println!("Version: {}", info.version);
        if let Some(s) = info.now_playing {
            println!(
                "Now playing:
  title: '{}'
  artist: '{}'
  album: '{}'
  track: '{}'
  disc: '{}'
  path: '{}'",
                s.title(),
                s.artist(),
                s.album(),
                s.track(),
                s.disc(),
                s.path().to_str().unwrap_or("?")
            )
        }
        if let Some(pos) = info.playlist_pos {
            println!("Playlist: {}/{}", pos, info.playlist_len);
        } else {
            println!("Playlist: ?/{}", info.playlist_len);
        }
        println!("Is playing: {}", info.is_playing);
        if let Some(ts) = info.timestamp {
            println!(
                "Timestamp: {}/{}",
                duration_to_string(ts.current, false),
                duration_to_string(ts.total, false),
            )
        } else {
            println!("Timestamp: ?/?")
        }
    }
}
