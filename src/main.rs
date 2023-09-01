use std::{
    env::{self, args},
    net::TcpStream,
    time::Duration,
};

use config::app_id;
use iced::{
    window::{self, PlatformSpecific},
    Application, Settings,
};
use log::{error, info};
use uamp_app::UampApp;

use crate::{
    arg_parser::{parse_args, Action},
    config::default_port,
    err::Result,
    messenger::Messenger,
};

mod arg_parser;
mod config;
mod err;
mod fancy_widgets;
mod library;
mod messenger;
mod player;
mod save_struct_macro;
mod song;
mod theme;
mod uamp_app;
mod uamp_gui;
mod wid;

fn main() -> Result<()> {
    if let Err(e) = start_logger() {
        eprintln!("failed to start logger: {e}");
    }

    info!("started");

    // on wayland, the app freezes when not drawn, this is temprary workaround
    // until it is fixed
    env::set_var("WINIT_UNIX_BACKEND", "x11");

    let args: Vec<_> = args().collect();
    let args = parse_args(args.iter().map(|a| a.as_ref()))?;

    for a in args.actions {
        match a {
            Action::Message(m) => {
                let res = send_message(m);
                if res.is_err() || !res.as_ref().unwrap().is_success() {
                    println!("Unexpected result: {res:?}");
                }
            }
        }
    }

    // should run has more power - in case both run and exit are true, run wins
    if args.should_run || !args.should_exit {
        if let Err(e) = UampApp::run(make_settings()) {
            error!("Uamp exited unexpectidly: {e}");
            Err(e)?;
        }
    }

    Ok(())
}

/// Creates the settings for the uamp app
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
        exit_on_close_request: false,
        ..Default::default()
    }
}

/// Tries to start the logger with env
///
/// # Errors
/// - cannot load from env
/// - cannot start the logger
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

/// Sends message to a existing uamp instance
fn send_message(msg: messenger::Message) -> Result<messenger::Message> {
    let stream = TcpStream::connect(format!("127.0.0.1:{}", default_port()))?;
    if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(5))) {
        error!("failed to send message: {}", e);
    }

    let mut msgr = Messenger::try_new(&stream)?;

    msgr.send(msg)?;

    msgr.recieve()
}
