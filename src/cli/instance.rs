use std::{mem, time::Instant};

use pareg::{Pareg, has_any_key, parse_arg};

use crate::core::{
    Result,
    config::Config,
    server::{ReqMsg, SndMsg, client::Client},
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
    pub messages: Vec<(SndMsg, Intention)>,
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
                        ReqMsg::Info(s.0, s.1).into(),
                        Intention::Default,
                    ))
                }
                v if has_any_key!(v, '=', "show") => {
                    let s = args
                        .cur_mval::<PlaylistRange>('=')?
                        .unwrap_or(PlaylistRange(1, 3));
                    self.messages.push((
                        ReqMsg::Info(s.0, s.1).into(),
                        Intention::Clear,
                    ))
                }
                v if has_any_key!(v, '=', "query", "list", "l") => {
                    self.messages.push((
                        ReqMsg::Query(args.cur_mval('=')?.unwrap_or_default())
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
                _ => self
                    .messages
                    .push((SndMsg::Ctrl(args.cur_arg()?), Intention::Default)),
            }
        }

        Ok(())
    }

    pub fn send(self, conf: &Config, props: &Props) -> Result<()> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()?;
        let local = tokio::task::LocalSet::new();
        rt.block_on(local.run_until(self.send_inner(conf, props)))
    }

    async fn send_inner(mut self, conf: &Config, props: &Props) -> Result<()> {
        let server = self
            .server
            .take()
            .unwrap_or_else(|| conf.server_address().to_owned());
        let port = self.port.unwrap_or(conf.port());

        let mut client = Client::connect(format!("{server}:{port}")).await?;

        // TODO: aggregate same type of messages

        for (m, i) in mem::take(&mut self.messages) {
            let send_time = Instant::now();
            match m {
                SndMsg::Ctrl(c) => client.send_ctrl(&[c]).await?,
                SndMsg::Req(ReqMsg::Info(b, a)) => {
                    let info = client.req_info(b, a).await?;
                    props.print_style.info(
                        &info,
                        conf,
                        props.color,
                        i == Intention::Clear,
                    );
                }
                SndMsg::Req(ReqMsg::Query(q)) => {
                    let songs = client.req_query(&q).await?;
                    props.print_style.song_list(
                        &songs,
                        &props.with_verbosity(self.verbosity),
                        send_time,
                    );
                }
            }
        }

        Ok(())
    }
}
