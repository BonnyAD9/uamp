use log::log;
use std::{fmt::Display, panic::Location};

#[track_caller]
pub fn log_result<T: Display>(level: log::Level, msg: &str, err: T) {
    let location = Location::caller();
    match msg {
        "" => log!(level, "{location}: {err:-}"),
        v if v.ends_with('.') => {
            log!(level, "{location}: {}: {err:-}", &msg[..msg.len() - 1])
        }
        v if v.ends_with(": ") => log!(level, "{location}: {msg}{err:-}"),
        v if v.ends_with(':') => log!(level, "{location}: {msg} {err:-}"),
        _ => log!(level, "{location}: {msg}: {err:-}"),
    }
}

pub trait LogResult<T>: Sized {
    type Success;

    #[track_caller]
    fn or_log(self, level: log::Level, msg: T) -> Option<Self::Success>;

    #[track_caller]
    fn or_log_err(self, msg: T) -> Option<Self::Success> {
        self.or_log(log::Level::Error, msg)
    }

    #[track_caller]
    fn or_warn(self, msg: T) -> Option<Self::Success> {
        self.or_log(log::Level::Warn, msg)
    }

    #[track_caller]
    fn _or_info(self, msg: T) -> Option<Self::Success> {
        self.or_log(log::Level::Info, msg)
    }
}

impl<T, E: Display, M: AsRef<str>> LogResult<M> for Result<T, E> {
    type Success = T;

    #[track_caller]
    fn or_log(self, level: log::Level, msg: M) -> Option<Self::Success> {
        match self {
            Ok(r) => Some(r),
            Err(e) => {
                log_result(level, msg.as_ref(), e);
                None
            }
        }
    }
}
