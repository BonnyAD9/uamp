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
    AudioTag(#[from] audiotags::Error),
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

/*
impl Error {
    pub fn invalid_operation() -> Self {
        Self::InvalidOperation("Invalid operation.".into())
    }

    pub fn unsupported() -> Self {
        Self::InvalidOperation("Not supported.".into())
    }

    pub fn no_stdin_pipe() -> Self {
        Self::InvalidOperation("No stdin pipe.".into())
    }

    pub fn invalid_value(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::InvalidValue(msg.into().into())
    }

    pub fn io(e: std::io::Error) -> Self {
        Self::Io(e.into())
    }

    pub fn no_color(self) -> Self {
        map_ctx!(self, |c| c.no_color(),
            Self::Pareg(p) => Self::Pareg(p.no_color()),
            Self::Multiple(mut m) => {
                for e in &mut m {
                    *e = mem::replace(e, Self::Multiple(vec![])).no_color();
                }
                Self::Multiple(m)
            }
        )
    }

    pub fn warn(self) -> Self {
        map_ctx!(self, |c| c.warn())
    }

    pub fn msg(self, msg: impl Into<Cow<'static, str>>) -> Self {
        map_ctx!(self, |c| c.msg(msg))
    }

    pub fn reason(self, reason: impl Into<Cow<'static, str>>) -> Self {
        map_ctx!(self, |c| c.reason(reason))
    }

    pub fn hint(self, hint: impl Into<Cow<'static, str>>) -> Self {
        map_ctx!(self, |c| c.hint(hint))
    }

    pub fn prepend(self, msg: impl Into<Cow<'static, str>>) -> Self {
        map_ctx!(self, |c| c.prepend(msg))
    }

    pub fn err<T>(self) -> Result<T> {
        Err(self)
    }

    pub fn log(self) -> Self {
        self.no_color()
    }

    pub fn clone_universal(&self) -> ErrCtx<String> {
        match self {
            Error::InvalidOperation(err_ctx) => err_ctx.clone_universal(),
            Error::ThreadPanicked(err_ctx) => err_ctx.clone_universal(),
            Error::NoStdinPipe(err_ctx) => err_ctx.clone_universal(),
            Error::ChildFailed(err_ctx) => err_ctx.clone_universal(),
            Error::NotFound(err_ctx) => err_ctx.clone_universal(),
            Error::Unexpected(err_ctx) => err_ctx.clone_universal(),
            Error::AudioTag(err_ctx) => err_ctx.clone_universal(),
            Error::Raplay(err_ctx) => err_ctx.clone_universal(),
            Error::SerdeJson(err_ctx) => err_ctx.clone_universal(),
            Error::Logger(err_ctx) => err_ctx.clone_universal(),
            Error::Io(err_ctx) => err_ctx.clone_universal(),
            Error::Time(err_ctx) => err_ctx.clone_universal(),
            Error::Notify(err_ctx) => err_ctx.clone_universal(),
            Error::ShellWords(err_ctx) => err_ctx.clone_universal(),
            Error::Other(err_ctx) => err_ctx.clone_universal(),
            Error::Http(err_ctx, _) => err_ctx.clone_universal(),
            Error::Url(err_ctx) => err_ctx.clone_universal(),
            Error::AddrParse(err_ctx) => err_ctx.clone_universal(),
            Error::Hyper(err_ctx) => err_ctx.clone_universal(),
            Error::HyperHttp(err_ctx) => err_ctx.clone_universal(),
            Error::Multiple(v) if v.len() == 1 => v[0].clone_universal(),
            Error::InvalidValue(v) => v.clone_universal(),
            Error::Libloading(v) => v.clone_universal(),
            e => e.to_string().into(),
        }
    }
}

macro_rules! impl_from {
    ($($et:ty => $en:ident),+ $(,)?) => {
        $(
            impl From<$et> for Error {
                fn from(value: $et) -> Self {
                    Self::$en(value.into())
                }
            }
        )+
    };
}

impl_from!(
    audiotags::Error => AudioTag,
    raplay::Error => Raplay,
    serde_json::Error => SerdeJson,
    std::io::Error => Io,
    SystemTimeError => Time,
    notify::Error => Notify,
    shell_words::ParseError => ShellWords,
    anyhow::Error => Other,
    image::ImageError => Image,
    url::ParseError => Url,
    std::net::AddrParseError => AddrParse,
    hyper::Error => Hyper,
    hyper::http::Error => HyperHttp,
    libloading::Error => Libloading,
);

#[cfg(unix)]
impl_from!(mpris_server::zbus::Error => Mpris);
*/
