use std::{
    env::args,
    net::TcpStream,
    time::Duration,
};

use config::app_id;
use eyre::{Report, Result};
use iced::{
    window::{self, PlatformSpecific},
    Application, Settings,
};
use log::{error, info};
use uamp_app::UampApp;

use crate::{config::default_port, messenger::Messenger};

mod config;
mod fancy_widgets;
mod library;
mod messenger;
mod player;
mod song;
mod theme;
mod uamp_app;
mod uamp_gui;
mod wid;

enum Action {
    Message(messenger::Message),
    Exit,
}

#[derive(Default)]
struct Args {
    actions: Vec<Action>,
}

fn main() -> Result<()> {
    if let Err(e) = start_logger() {
        eprintln!("failed to start logger: {e}");
    }

    info!("started");

    let args: Vec<_> = args().collect();
    let args = parse_args(args.iter().map(|a| a.as_ref()))?;

    if args.actions.len() == 0 {
        UampApp::run(make_settings())?;
        return Ok(());
    }

    for a in args.actions {
        match a {
            Action::Message(m) => {
                let res = send_message(m);
                if res.is_err() || !res.as_ref().unwrap().is_success() {
                    println!("Unexpected result: {res:?}");
                }
            },
            Action::Exit => return Ok(()),
        }
    }

    Ok(())
}

fn make_settings() -> Settings<()> {
    let icon = window::icon::from_rgba(
        include_bytes!("../assets/raw_img/icon_64.data")
            .to_owned()
            .into(),
        64,
        64,
    );

    if let Err(e) = &icon {
        error!("Failed to set the icon: {e}");
    }

    Settings {
        window: window::Settings {
            icon: icon.ok(),
            platform_specific: PlatformSpecific {
                application_id: app_id(),
            },
            ..Default::default()
        },
        id: Some(app_id()),
        ..Default::default()
    }
}

fn start_logger() -> Result<()> {
    flexi_logger::Logger::try_with_env()?
        .log_to_file(
            flexi_logger::FileSpec::default()
                .directory(config::default_config_path().join("log")),
        )
        .write_mode(flexi_logger::WriteMode::BufferAndFlush)
        .start()?;
    Ok(())
}

fn parse_args<'a>(args: impl Iterator<Item = &'a str>) -> Result<Args> {
    let mut res = Args::default();

    macro_rules! ret_err {
        ($($arg:tt)*) => {
            return Err(Report::msg(format!($($arg)*)))
        };
    }

    macro_rules! next {
        ($iter:ident, $after:literal) => {
            match $iter.next() {
                Some(a) => a,
                None => {
                    ret_err!("Expected instance action after '{}'", $after)
                }
            }
        }
    }

    let mut args = args.skip(1);

    while let Some(a) = args.next() {
        match a {
            "i" | "instance" => {
                let a = next!(args, "instance");

                match a {
                    "play-pause" | "pp" => res.actions.push(Action::Message(
                        messenger::Message::Control(
                            messenger::Control::PlayPause,
                        ),
                    )),
                    _ => ret_err!("Invalid argument '{a}' after 'instance'"),
                }
            }
            "h" | "help" | "-h" | "--help" | "-?" => {
                help();
                res.actions.push(Action::Exit);
            }
            a => ret_err!("Invalid argument '{a}'"),
        }
    }

    Ok(res)
}

fn send_message(msg: messenger::Message) -> Result<messenger::Message> {
    let stream =
        TcpStream::connect(format!("127.0.0.1:{}", default_port()))?;
    _ = stream.set_read_timeout(Some(Duration::from_secs(5)));

    let mut msgr = Messenger::try_new(&stream)?;

    msgr.send(msg)?;

    msgr.recieve()
}

fn help() {
    println!("Welcome in uamp by BonnyAD9
Version 0.1.0

Usage:
  uamp
    starts the gui of the player

  uamp [action]
    does the given action

Actions:
  i instance [instance action]
    operates on a running instance of uamp

  h help -h -? --help
    shows this help

Instance actions:
  pp play-pause
    toggle between the states playing and paused")
}
