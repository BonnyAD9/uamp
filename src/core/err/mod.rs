use std::{any::Any, borrow::Cow, fmt::Display, mem, time::SystemTimeError};

use flexi_logger::FlexiLoggerError;
use itertools::Itertools;
use log::error;
use thiserror::Error;

mod err_ctx;
mod err_ctx_flags;

pub use self::{err_ctx::*, err_ctx_flags::*};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Result with the unified error type of uamp
pub type Result<T> = std::result::Result<T, Error>;

/// Unified error type of uamp
#[derive(Error, Debug)]
pub enum Error {
    /// The requested operatoin was invalid at the time.
    #[error("{0}")]
    InvalidOperation(Box<ErrCtx<&'static str>>),
    /// A secondary thread panicked.
    #[error("{0}")]
    ThreadPanicked(Box<ErrCtx<&'static str>>),
    /// Cannot pipe to stdin of child process.
    #[error("{0}")]
    NoStdinPipe(Box<ErrCtx<&'static str>>),
    /// Child process exit with failure.
    #[error("{0}")]
    ChildFailed(Box<ErrCtx<String>>),
    /// Something is not present.
    #[error("{0}")]
    NotFound(Box<ErrCtx<&'static str>>),
    /// An unexpected error.
    #[error("{0}")]
    Unexpected(Box<ErrCtx<&'static str>>),
    /// Invalid value.
    #[error("{0}")]
    InvalidValue(Box<ErrCtx<Cow<'static, str>>>),
    /// Failed to parse arguments.
    #[error(transparent)]
    Pareg(#[from] pareg::ArgError),
    /// the audiotags library error.
    #[error("{0}")]
    AudioTag(Box<ErrCtx<audiotags::Error>>),
    /// The raplay library returned error.
    #[error("{0}")]
    Raplay(Box<ErrCtx<raplay::Error>>),
    /// Serde json error
    #[error("{0}")]
    SerdeJson(Box<ErrCtx<serde_json::Error>>),
    /// The logger returned error (oops :|| ).
    #[error("{0}")]
    Logger(Box<ErrCtx<FlexiLoggerError>>),
    /// Some standard library io error.
    #[error("{0}")]
    Io(Box<ErrCtx<std::io::Error>>),
    /// Time doesn't work :||
    #[error("{0}")]
    Time(Box<ErrCtx<SystemTimeError>>),
    /// Errors from the notify library.
    #[error("{0}")]
    Notify(Box<ErrCtx<notify::Error>>),
    /// Errors from the library shell_words.
    #[error("{0}")]
    ShellWords(Box<ErrCtx<shell_words::ParseError>>),
    /// Error from reading image.
    #[error("{0}")]
    Image(Box<ErrCtx<image::ImageError>>),
    /// Error from mpris
    #[cfg(unix)]
    #[error("{0}")]
    Mpris(Box<ErrCtx<mpris_server::zbus::Error>>),
    /// Any other error.
    #[error("{0}")]
    Other(Box<ErrCtx<anyhow::Error>>),
    /// Error when handling HTTP request.
    #[error("{1} {0}")]
    Http(Box<ErrCtx<String>>, u16),
    #[error("{0}")]
    Url(Box<ErrCtx<url::ParseError>>),
    #[error("{0}")]
    AddrParse(Box<ErrCtx<std::net::AddrParseError>>),
    #[error("{0}")]
    Hyper(Box<ErrCtx<hyper::Error>>),
    #[error("{0}")]
    HyperHttp(Box<ErrCtx<hyper::http::Error>>),
    #[error("{}", .0.iter().map(|a| a.to_string()).join(""))]
    Multiple(Vec<Error>),
}

macro_rules! map_ctx {
    ($s:ident, |$ctx:ident| $f:expr $(, $($p:pat => $pb:expr),* $(,)?)?) => {
        match $s {
            Error::InvalidOperation(mut $ctx) => {
                *$ctx = $f;
                Error::InvalidOperation($ctx)
            }
            Error::ThreadPanicked(mut $ctx) => {
                *$ctx = $f;
                Error::ThreadPanicked($ctx)
            }
            Error::NoStdinPipe(mut $ctx) => {
                *$ctx = $f;
                Error::NoStdinPipe($ctx)
            }
            Error::ChildFailed(mut $ctx) => {
                *$ctx = $f;
                Error::ChildFailed($ctx)
            }
            Error::NotFound(mut $ctx) => {
                *$ctx = $f;
                Error::NotFound($ctx)
            }
            Error::Unexpected(mut $ctx) => {
                *$ctx = $f;
                Error::Unexpected($ctx)
            }
            Error::AudioTag(mut $ctx) => {
                *$ctx = $f;
                Error::AudioTag($ctx)
            }
            Error::SerdeJson(mut $ctx) => {
                *$ctx = $f;
                Error::SerdeJson($ctx)
            }
            Error::Logger(mut $ctx) => {
                *$ctx = $f;
                Error::Logger($ctx)
            }
            Error::Io(mut $ctx) => {
                *$ctx = $f;
                Error::Io($ctx)
            }
            Error::Time(mut $ctx) => {
                *$ctx = $f;
                Error::Time($ctx)
            }
            Error::Notify(mut $ctx) => {
                *$ctx = $f;
                Error::Notify($ctx)
            }
            Error::ShellWords(mut $ctx) => {
                *$ctx = $f;
                Error::ShellWords($ctx)
            }
            Error::Image(mut $ctx) => {
                *$ctx = $f;
                Error::Image($ctx)
            }
            Error::Http(mut $ctx, code) => {
                *$ctx = $f;
                Error::Http($ctx, code)
            }
            Error::Url(mut $ctx) => {
                *$ctx = $f;
                Error::Url($ctx)
            }
            Error::AddrParse(mut $ctx) => {
                *$ctx = $f;
                Error::AddrParse($ctx)
            }
            Error::Hyper(mut $ctx) => {
                *$ctx = $f;
                Error::Hyper($ctx)
            }
            Error::HyperHttp(mut $ctx) => {
                *$ctx = $f;
                Error::HyperHttp($ctx)
            }
            Error::InvalidValue(mut $ctx) => {
                *$ctx = $f;
                Error::InvalidValue($ctx)
            }
            $($($p => $pb,)*)?
            e => e,
        }
    };
}

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

    pub fn http(code: u16, msg: String) -> Self {
        Self::Http(msg.into(), code)
    }

    pub fn thread_panicked(e: Option<Box<dyn Any + Send + 'static>>) -> Self {
        let res = Self::ThreadPanicked("A spawned thread panicked.".into());

        let reason: Option<Cow<'static, str>> = e.and_then(|e| {
            e.downcast::<&'static str>()
                .map(|s| (*s).into())
                .or_else(|e| e.downcast::<String>().map(|s| (*s).into()))
                .ok()
        });

        if let Some(reason) = reason {
            res.reason(reason)
        } else {
            res
        }
    }

    pub fn io(e: std::io::Error) -> Self {
        Self::Io(e.into())
    }

    pub fn multiple(mut e: Vec<Error>) -> Result<()> {
        match e.len() {
            0 => Ok(()),
            1 => Err(e.pop().unwrap()),
            _ => Err(Error::Multiple(
                e.into_iter().fold(vec![], |a, e| e.add_to(a)),
            )),
        }
    }

    fn add_to(self, mut r: Vec<Error>) -> Vec<Error> {
        if let Self::Multiple(m) = self {
            for e in m {
                r = e.add_to(r);
            }
        } else {
            r.push(self);
        }
        r
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
            e => e.to_string().into(),
        }
    }
}

pub fn log_err<T, E: Display>(
    pf: &str,
    e: std::result::Result<T, E>,
) -> Option<T> {
    match e {
        Ok(v) => Some(v),
        Err(e) => {
            error!("{pf}{e}");
            None
        }
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(value: tokio::task::JoinError) -> Self {
        if value.is_panic() {
            Self::thread_panicked(Some(value.into_panic()))
        } else {
            Self::Unexpected("Task was stopped.".into())
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
);

#[cfg(unix)]
impl_from!(mpris_server::zbus::Error => Mpris);
