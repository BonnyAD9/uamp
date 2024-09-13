use std::{
    borrow::Cow,
    mem,
    net::TcpStream,
    path::Path,
    time::{Duration, Instant},
};

use log::error;
use pareg::{has_any_key, ArgIterator, ByRef};
use termal::{eprintacln, printcln, printmc, printmcln};

use crate::{
    core::{
        config::Config,
        library::Song,
        messenger::{DataResponse, Info, Messenger, MsgMessage, Request},
        PlayMsg, Result,
    },
    ext::duration_to_string,
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
        color: bool,
    ) -> Result<()>
    where
        I: Iterator,
        I::Item: ByRef<&'a str>,
    {
        while let Some(arg) = args.next() {
            match arg {
                "info" | "nfo" => {
                    self.messages.push(Request::Info(1, 3).into())
                }
                v if has_any_key!(v, '=', "p", "play") => {
                    self.messages.push(
                        PlayMsg::TmpPath(
                            args.cur_val::<&Path>('=')?.canonicalize()?.into(),
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
                "-p" | "--port" => self.port = Some(args.next_arg()?),
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
                Err(e) => eprintacln!("{'r}error: {'_}{e}"),
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
        ))?;
        if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(5))) {
            error!("failed to send message: {}", e);
        }

        let mut msgr = Messenger::new(&stream);

        msgr.send(msg)?;

        msgr.recieve()
    }

    fn print_data(data: DataResponse, color: bool, send_time: Instant) {
        match data {
            DataResponse::Info(i) => Self::print_info(*i, color),
            DataResponse::SongList(songs) => {
                Self::print_list(songs, color, send_time)
            }
        }
    }

    fn print_info(info: Info, color: bool) {
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
            pos = (p + 1).to_string().into();
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

        let blen = 80_usize.saturating_sub(
            artist.chars().count() + album.chars().count() + 9,
        ) / 2;
        let playlist = format!("{pos}/{plen}");
        let dt = format!("<{disc}-{track}>");

        printmcln!(
            color,
            "
{'bold y}{title: ^80}{'_}
{: >blen$}{'gr}by {'dc}{artist} {'gr}from {'dm}{album}{'_}

     {'w}{cur: <27}{'_ bold}<||    {'y}{state}    {'_fg}||>{'_ w}{total: >27}{'_}
    {'bold}[{'_ y}{before}{'w bold}{is}{'_ gr}{after}{'_ bold}]{'_}

{'gr}{playlist: ^80}
{dt: ^80}{'_}",
            ' ',
        );

        if info.before.is_empty() && info.after.is_empty() {
            printmcln!(color, "{'gr}uamp{version: >76}{'_}");
            return;
        }

        printmcln!(color, "\n{'gr}{: ^80}{'_}", "----<< playlist >>----");
        if info
            .playlist_pos
            .map(|p| p - info.before.len() != 0)
            .unwrap_or(true)
        {
            printmcln!(color, "{'gr}{: ^80}", "...");
        } else {
            println!();
        }

        for (i, s) in info.before.iter().enumerate() {
            if let Some(idx) = info.playlist_pos {
                let idx = idx + i - before.len();
                printmcln!(
                    color,
                    "  {'gr}{idx}. {'dc}{} {'gr}- {'dy}{}{'_}",
                    s.artist(),
                    s.title()
                );
            } else {
                printmcln!(
                    color,
                    "  {'dc}{} {'gr}- {'dy}{}{'_}",
                    s.artist(),
                    s.title()
                );
            }
        }
        if let Some(idx) = info.playlist_pos {
            printmcln!(
                color,
                "  {'_}{}. {'c}{artist} {'_}- {'y}{title}{'_}",
                idx + 1
            );
        } else {
            printmcln!(color, "  {'c}{artist} {'_}- {'y}{title}{'_}");
        }
        for (i, s) in info.after.iter().enumerate() {
            if let Some(idx) = info.playlist_pos {
                let idx = idx + i + 2;
                printmcln!(
                    color,
                    "  {'gr}{idx}. {'dc}{} {'gr}- {'dy}{}{'_}",
                    s.artist(),
                    s.title()
                );
            } else {
                printmcln!(
                    color,
                    "  {'dc}{} {'gr}- {'dy}{}{'_}",
                    s.artist(),
                    s.title()
                );
            }
        }

        if info
            .playlist_pos
            .map(|p| p + info.after.len() + 1 < info.playlist_len)
            .unwrap_or(true)
        {
            printmcln!(color, "{'gr}{: ^80}", "...");
        } else {
            println!();
        }

        printmcln!(color, "{'gr}uamp{version: >76}{'_}");
    }

    fn print_list(songs: Vec<Song>, color: bool, send_time: Instant) {
        printmcln!(
            color,
            "{'bold y}{:<30} {'c}{:<20} {'m}{:28}{'_}",
            "Title",
            "Artist",
            "Album"
        );
        let mut total_dur = Duration::ZERO;
        for s in &songs {
            total_dur += s.length();
            Self::print_song(s, color);
        }

        let elapsed = Instant::now() - send_time;

        printcln!(
            "{'gr}{} songs ({}) in {:.4} s",
            songs.len(),
            duration_to_string(total_dur, true),
            elapsed.as_secs_f32(),
        );
    }

    fn print_song(s: &Song, color: bool) {
        printmc!(color, "{'dy}");
        Self::print_elipsised(s.title(), 30);
        printmc!(color, " {'dc}");
        Self::print_elipsised(s.artist(), 20);
        printmc!(color, " {'dm}");
        Self::print_elipsised(s.album(), 28);
        printmcln!(color, "{'_}");
    }

    fn print_elipsised(s: &str, len: usize) {
        let mut ind = s.char_indices();
        let Some((i, _)) = ind.nth(len - 3) else {
            print!("{s:<len$}");
            return;
        };

        if ind.nth(3).is_some() {
            print!("{}...", &s[..i]);
        } else {
            print!("{s:<len$}");
        }
    }
}
