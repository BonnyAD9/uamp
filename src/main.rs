use core::{Error, config};
use std::{
    backtrace::Backtrace,
    io::{self, IsTerminal},
    panic::{self, PanicHookInfo},
    process::ExitCode,
};

use cli::Run;
use flexi_logger::LoggerHandle;
use log::{error, info};
use pareg::Pareg;
use termal::{eprintacln, eprintmcln};

use crate::{
    cli::{Action, Args},
    core::Result,
};

mod background_app;
mod cli;
mod core;
mod env;
mod ext;

fn main() -> ExitCode {
    let _ = match start_logger() {
        Err(e) => {
            eprintmcln!(io::stderr().is_terminal(), "{e}");
            None
        }
        Ok(l) => Some(l),
    };
    register_panic_hook();

    match start() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintacln!("{e}");
            // Don't log errors that are clearly displayed to user.
            // error!("main returns error:\n{}", e.log());
            ExitCode::FAILURE
        }
    }
}

/// Main wraps this function, this is the entry point of the application
fn start() -> Result<()> {
    info!("started");

    let args = Args::parse(Pareg::args())?;

    let conf = args.make_config();

    for a in args.actions {
        match a {
            Action::Instance(i) => i.send(&conf, &args.props)?,
            Action::RunDetached(mut i) => {
                i.port = i.port.or(args.port);
                i.server_address = i
                    .server_address
                    .take()
                    .or_else(|| args.server_address.to_owned());
                i.run_detached()?;
            }
            Action::Config(c) => c.act()?,
            Action::Shell(s) => s.act(),
            Action::Internal(i) => i.act(&conf)?,
            Action::Man(m) => m.act()?,
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
fn start_logger() -> Result<LoggerHandle> {
    flexi_logger::Logger::try_with_env_or_str("warn")
        .map_err(|e| {
            Error::Logger(e.into()).msg("Failed to initialize logger.")
        })?
        .log_to_file(
            flexi_logger::FileSpec::default()
                .directory(config::default_log_dir()),
        )
        .write_mode(flexi_logger::WriteMode::Direct)
        .start()
        .map_err(|e| Error::Logger(e.into()).msg("Failed to start logger."))
}

fn register_panic_hook() {
    let hook = panic::take_hook();
    let hook = move |phi: &PanicHookInfo| {
        let payload = phi.payload();
        let loc = phi
            .location()
            .map(|l| format!(" at {l}"))
            .unwrap_or_default();

        // Panicking is not something that should ever happen. We can afford to
        // capture the full backtrace if it will help resolve the panic.
        let bt = Backtrace::force_capture();

        if let Some(s) = payload.downcast_ref::<String>() {
            error!("panicking{loc}: {s}\nbacktrace:\n{bt}");
        } else if let Some(s) = payload.downcast_ref::<&'static str>() {
            error!("panicking{loc}: {s}\nbacktrace:\n{bt}");
        } else {
            error!("panicking with unknown message{loc}.\nbacktrace:\n{bt}");
        }

        // Chain the original hook.
        hook(phi)
    };

    panic::set_hook(Box::new(hook));
}
