use std::{any::Any, borrow::Cow, mem, time::SystemTimeError};

use flexi_logger::FlexiLoggerError;
use itertools::Itertools;
use log::error;
use pareg::ColorMode;
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
    /// Cannot get my own name.
    #[error("{0}")]
    NoProgramName(Box<ErrCtx<&'static str>>),
    /// The requested operatoin was invalid at the time.
    #[error("{0}")]
    InvalidOperation(Box<ErrCtx<&'static str>>),
    /// A secondary thread panicked.
    #[error("{0}")]
    ThreadPanicked(Box<ErrCtx<&'static str>>),
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
    /// Rmp error while encoding
    #[error("{0}")]
    SerdeRmpEncode(Box<ErrCtx<rmp_serde::encode::Error>>),
    /// Rmp error while decoding
    #[error("{0}")]
    SerdeRmpDecode(Box<ErrCtx<rmp_serde::decode::Error>>),
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
    /// Any other error.
    #[error("{0}")]
    Other(Box<ErrCtx<anyhow::Error>>),
    #[error("{}", .0.iter().map(|a| a.to_string()).join(""))]
    Multiple(Vec<Error>),
}

macro_rules! map_ctx {
    ($s:ident, |$ctx:ident| $f:expr $(, $($p:pat => $pb:expr),* $(,)?)?) => {
        match $s {
            Error::NoProgramName(mut $ctx) => {
                *$ctx = $f;
                Error::NoProgramName($ctx)
            }
            Error::InvalidOperation(mut $ctx) => {
                *$ctx = $f;
                Error::InvalidOperation($ctx)
            }
            Error::AudioTag(mut $ctx) => {
                *$ctx = $f;
                Error::AudioTag($ctx)
            }
            Error::SerdeJson(mut $ctx) => {
                *$ctx = $f;
                Error::SerdeJson($ctx)
            }
            Error::SerdeRmpDecode(mut $ctx) => {
                *$ctx = $f;
                Error::SerdeRmpDecode($ctx)
            }
            Error::SerdeRmpEncode(mut $ctx) => {
                *$ctx = $f;
                Error::SerdeRmpEncode($ctx)
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
            $($($p => $pb,)*)?
            e => e,
        }
    };
}

impl Error {
    pub fn no_program_name() -> Self {
        Self::NoProgramName("Cannot get path to uamp binary.".into())
    }

    pub fn invalid_operation() -> Self {
        Self::InvalidOperation("Invalid operation.".into())
    }

    pub fn unsupported() -> Self {
        Self::InvalidOperation("Not supported.".into())
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
            Self::Pareg(p) => Self::Pareg(p.map_ctx(|mut c| {
                c.color = ColorMode::Never;
                c
            })),
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
            Error::NoProgramName(err_ctx) => err_ctx.clone_universal(),
            Error::InvalidOperation(err_ctx) => err_ctx.clone_universal(),
            Error::ThreadPanicked(err_ctx) => err_ctx.clone_universal(),
            Error::AudioTag(err_ctx) => err_ctx.clone_universal(),
            Error::Raplay(err_ctx) => err_ctx.clone_universal(),
            Error::SerdeJson(err_ctx) => err_ctx.clone_universal(),
            Error::SerdeRmpEncode(err_ctx) => err_ctx.clone_universal(),
            Error::SerdeRmpDecode(err_ctx) => err_ctx.clone_universal(),
            Error::Logger(err_ctx) => err_ctx.clone_universal(),
            Error::Io(err_ctx) => err_ctx.clone_universal(),
            Error::Time(err_ctx) => err_ctx.clone_universal(),
            Error::Notify(err_ctx) => err_ctx.clone_universal(),
            Error::ShellWords(err_ctx) => err_ctx.clone_universal(),
            Error::Other(err_ctx) => err_ctx.clone_universal(),
            Error::Multiple(v) if v.len() == 1 => v[0].clone_universal(),
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
    // rmp_serde::encode::Error => SerdeRmpEncode,
    // rmp_serde::decode::Error => SerdeRmpDecode,
    std::io::Error => Io,
    SystemTimeError => Time,
    notify::Error => Notify,
    shell_words::ParseError => ShellWords,
    anyhow::Error => Other,
);