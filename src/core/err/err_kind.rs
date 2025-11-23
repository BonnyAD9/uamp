use std::{
    backtrace::Backtrace, borrow::Cow, process::ExitStatus,
    time::SystemTimeError,
};

use flexi_logger::FlexiLoggerError;
use itertools::Itertools;
use thiserror::Error;

use crate::ext::Wrap;

/// Unified error type of uamp
#[derive(Error, Debug)]
pub enum ErrKind {
    /// The requested operatoin was invalid at the time.
    #[error("Invalid operation.")]
    InvalidOperation,
    #[error("Unsupported.")]
    Unsupported,
    /// A secondary thread panicked.
    #[error("Thread panicked: {0}")]
    ThreadPanicked(Cow<'static, str>),
    /// Cannot pipe to stdin of child process.
    #[error("Failed to pipe stdin to child process.")]
    NoStdinPipe,
    /// Child process exit with failure.
    #[error(
        "Child process failed with code {code}.{}",
        if let Some(v) = stderr {
            format!(" stderr: \n{v}")
        } else {
            "".to_owned()
        }
    )]
    ChildFailed {
        code: ExitStatus,
        stderr: Option<Cow<'static, str>>,
    },
    /// Something is not present.
    #[error("Not found.")]
    NotFound,
    /// An unexpected error.
    #[error("An unexpected error at: {}.", .0.0)]
    Unexpected(Wrap<Backtrace>),
    /// Invalid value.
    #[error("Invalid value.")]
    InvalidValue,
    /// Failed to parse arguments.
    #[error(transparent)]
    Pareg(#[from] pareg::ArgError),
    /// the audiotags library error.
    #[error(transparent)]
    Ratag(#[from] ratag::Error),
    /// The raplay library returned error.
    #[error(transparent)]
    Raplay(#[from] raplay::Error),
    /// Serde json error
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    /// The logger returned error (oops :|| ).
    #[error(transparent)]
    Logger(#[from] FlexiLoggerError),
    /// Some standard library io error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Time doesn't work :||
    #[error(transparent)]
    Time(#[from] SystemTimeError),
    /// Errors from the notify library.
    #[error(transparent)]
    Notify(#[from] notify::Error),
    /// Errors from the library shell_words.
    #[error(transparent)]
    ShellWords(#[from] shell_words::ParseError),
    /// Error from reading image.
    #[error(transparent)]
    Image(#[from] image::ImageError),
    /// Error from mpris
    #[cfg(unix)]
    #[error(transparent)]
    Mpris(#[from] mpris_server::zbus::Error),
    /// Any other error.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    /// Error when handling HTTP request.
    #[error("{0} {1}")]
    Http(u16, Cow<'static, str>),
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error(transparent)]
    AddrParse(#[from] std::net::AddrParseError),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    HyperHttp(#[from] hyper::http::Error),
    #[error("{}", .0.iter().map(|a| a.to_string()).join("\n"))]
    Multiple(Vec<super::Error>),
    #[error(transparent)]
    Libloading(#[from] libloading::Error),
}
