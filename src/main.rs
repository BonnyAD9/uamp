use std::env::args;

use cli::Run;
use log::info;

use crate::{
    cli::{Action, Args},
    core::Result,
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
            Action::Instance(i) => i.send(&conf)?,
            Action::RunDetached(mut i) => {
                i.port = i.port.or(args.port);
                i.server_address = i
                    .server_address
                    .take()
                    .or_else(|| args.server_address.to_owned());
                i.run_detached()?;
            }
        }
    }

    if let Some(info) = args.run {
        info.run_app(conf)?;
    } else if !args.should_exit {
        Run::default().run_app(conf)?;
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
