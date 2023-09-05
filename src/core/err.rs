use flexi_logger::FlexiLoggerError;
use log::error;
use thiserror::Error;

use crate::{
    cli::CliError,
    hotkeys::{self, HotkeyError},
};

/// Result with the unified error type of uamp
pub type Result<T> = std::result::Result<T, Error>;

/// Unified error type of uamp
#[derive(Error, Debug)]
pub enum Error {
    /// The requested operatoin was invalid at the time
    #[error("Operation is invalid: {0}")]
    InvalidOperation(&'static str),
    /// A secondary thread panicked
    #[error("A spawned thread panicked")]
    ThreadPanicked,
    /// Failed to parse arguments
    #[error(transparent)]
    ArgParse(#[from] CliError),
    /// Failed to register some or all hotkeys
    #[error(transparent)]
    Hotkey(#[from] HotkeyError),
    /// The audio tag library returned error
    #[error(transparent)]
    AudioTag(#[from] audiotags::Error),
    /// The raplay library returned error
    #[error(transparent)]
    Raplay(#[from] raplay::Error),
    /// The serde library returned error
    #[error(transparent)]
    Serde(#[from] SerdeError),
    /// The logger returned error (oops :|| )
    #[error(transparent)]
    Logger(#[from] FlexiLoggerError),
    /// Iced library returned error
    #[error(transparent)]
    Iced(#[from] iced::Error),
    /// Some standard library io error
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Any other error
    #[error(transparent)]
    Other(anyhow::Error),
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
    global_hotkey::Error => Hotkey,
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
