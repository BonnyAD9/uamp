use flexi_logger::FlexiLoggerError;
use log::error;
use thiserror::Error;

use crate::arg_parser;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ArgParse(#[from] arg_parser::Error),
    #[error(transparent)]
    Hotkey(#[from] global_hotkey::Error),
    #[error(transparent)]
    AudioTag(#[from] audiotags::Error),
    #[error(transparent)]
    Raplay(#[from] raplay::Error),
    #[error(transparent)]
    Serde(#[from] SerdeError),
    #[error(transparent)]
    Logger(#[from] FlexiLoggerError),
    #[error(transparent)]
    Iced(#[from] iced::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
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
);

#[derive(Error, Debug)]
pub enum SerdeError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    RmpEncode(#[from] rmp_serde::encode::Error),
    #[error(transparent)]
    RmpDecode(#[from] rmp_serde::decode::Error),
}
