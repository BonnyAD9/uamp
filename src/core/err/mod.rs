use std::{borrow::Cow, time::SystemTimeError};

use flexi_logger::FlexiLoggerError;
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
    /// Cannot get my own name.
    #[error("{0}")]
    NoProgramName(Box<ErrCtx<&'static str>>),
    /// The requested operatoin was invalid at the time.
    #[error("{0}")]
    InvalidOperation(Box<ErrCtx<&'static str>>),
    /// A secondary thread panicked.
    #[error("A spawned thread panicked")]
    ThreadPanicked,
    /// Failed to parse integer from string.
    #[error(transparent)]
    IntParse(#[from] std::num::ParseIntError),
    /// Failed to parse arguments.
    #[error(transparent)]
    Pareg(#[from] pareg::ArgError),
    /// the audiotags library error.
    #[error(transparent)]
    AudioTag(#[from] audiotags::Error),
    /// The raplay library returned error.
    #[error(transparent)]
    Raplay(#[from] raplay::Error),
    /// The serde library returned error.
    #[error(transparent)]
    Serde(#[from] SerdeError),
    /// The logger returned error (oops :|| ).
    #[error(transparent)]
    Logger(#[from] FlexiLoggerError),
    /// Some standard library io error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Time dowsn't work :||
    #[error(transparent)]
    Time(#[from] SystemTimeError),
    /// Synchronization error :|| Shouldn't happen.
    #[error("Failed to lock: {0}")]
    Poison(String),
    /// Errors from the notify library.
    #[error(transparent)]
    Notify(#[from] notify::Error),
    /// Errors from the library shell_words.
    #[error(transparent)]
    ShellWords(#[from] shell_words::ParseError),
    /// Any other error.
    #[error(transparent)]
    Other(anyhow::Error),
}

macro_rules! map_ctx {
    ($s:ident, |$ctx:ident| $f:expr) => {
        match $s {
            Error::NoProgramName(mut $ctx) => {
                *$ctx = $f;
                Error::NoProgramName($ctx)
            }
            Error::InvalidOperation(mut $ctx) => {
                *$ctx = $f;
                Error::InvalidOperation($ctx)
            }
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

    pub fn no_color(self) -> Self {
        map_ctx!(self, |c| c.no_color())
    }

    pub fn msg(self, msg: impl Into<Cow<'static, str>>) -> Self {
        map_ctx!(self, |c| c.msg(msg))
    }

    pub fn reason(self, reason: impl Into<Cow<'static, str>>) -> Self {
        map_ctx!(self, |c| c.reason(reason))
    }

    pub fn err<T>(self) -> Result<T> {
        Err(self)
    }

    pub fn log(self) -> Self {
        self.no_color()
    }
}

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        if value.is::<Self>() {
            value.downcast().unwrap()
        } else {
            Self::Other(value)
        }
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(value: std::sync::PoisonError<T>) -> Self {
        Self::Poison(value.to_string())
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
    serde_json::Error => Serde,
    rmp_serde::encode::Error => Serde,
    rmp_serde::decode::Error => Serde,
);

/// Collections of errors while serializing
#[derive(Error, Debug)]
pub enum SerdeError {
    /// Serde json error
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Rmp error while encoding
    #[error(transparent)]
    RmpEncode(#[from] rmp_serde::encode::Error),
    /// Rmp error while decoding
    #[error(transparent)]
    RmpDecode(#[from] rmp_serde::decode::Error),
}
