use core::Error;
use std::{
    env::{self, args},
    net::TcpStream,
    process::{Command, Stdio},
    time::Duration,
};

use cli::RunInfo;
use config::Config;
use log::{error, info, warn};

use crate::{
    background_app::run_background_app,
    cli::{Action, Args},
    core::{
        extensions::duration_to_string,
        messenger::{self, msg::Info, Messenger},
        Result,
    },
    messenger::{msg, MsgMessage},
};

mod app;
mod background_app;
mod cli;
mod config;
mod core;
mod library;
mod player;
mod sync;

fn main() {
    if let Err(e) = start() {
        println!("{e}");
    }
}

/// Main wraps this function, this is the entry point of the application
fn start() -> Result<()> {
    if let Err(e) = start_logger() {
        eprintln!("failed to start logger: {e}");
    }

    info!("started");

    let args: Vec<_> = args().collect();
    let args = Args::parse(args.iter().into())?;

    let conf = args.make_config();

    for a in args.actions {
        match a {
            Action::Message(m) => {
                for m in m {
                    let res = send_message(&conf, m);
                    match res {
                        Ok(MsgMessage::Success) => {}
                        Ok(MsgMessage::Info(i)) => {
                            print_info(*i);
                        }
                        Err(e) => println!("{e}"),
                        Ok(r) => {
                            println!("Unexpected response: {r:?}");
                        }
                    }
                }
            }
            Action::RunDetached(mut i) => {
                i.port = i.port.or(args.port);
                i.server_address = i
                    .server_address
                    .take()
                    .or_else(|| args.server_address.to_owned());
                run_detached(i)?;
            }
        }
    }

    if let Some(info) = args.run {
        run_app(info, conf)?;
    } else if !args.should_exit {
        run_app(RunInfo::default(), conf)?;
    }

    Ok(())
}

/// Tries to start the logger with env
///
/// # Errors
/// - cannot load from env
/// - cannot start the logger
fn start_logger() -> Result<()> {
    match flexi_logger::Logger::try_with_env_or_str("warn") {
        Ok(l) => l,
        Err(_) => flexi_logger::Logger::try_with_str("warn")?,
    }
    .log_to_file(
        flexi_logger::FileSpec::default()
            .directory(config::default_config_path().join("log")),
    )
    .write_mode(flexi_logger::WriteMode::BufferAndFlush)
    .start()?;
    Ok(())
}

/// Sends message to a existing uamp instance
fn send_message(conf: &Config, msg: msg::Message) -> Result<msg::Message> {
    let stream = TcpStream::connect(format!(
        "{}:{}",
        conf.server_address(),
        conf.port()
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

fn run_app(info: RunInfo, mut conf: Config) -> Result<()> {
    if info.detach {
        warn!("Detach is set to detached when not running as detached.");
    }

    info.update_config(&mut conf);
    run_background_app(conf)
}

fn run_detached(info: RunInfo) -> Result<()> {
    if !info.detach {
        warn!("Detached is not set to detached when running as detached");
    }

    let cmd = env::args_os().next().ok_or(Error::NoProgramName)?;
    let mut cmd = Command::new(cmd);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());

    cmd.arg("run");

    if let Some(p) = info.port {
        cmd.args(["-p", &p.to_string()]);
    }
    if let Some(a) = info.server_address {
        cmd.args(["-a", &a]);
    }

    let child = cmd.spawn()?;
    let id = child.id();
    println!("Spawned detached process with id {id}");
    info!("Spawned detached process with id {id}");

    Ok(())
}
