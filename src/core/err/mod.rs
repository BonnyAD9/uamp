use std::{
    any::Any,
    backtrace::Backtrace,
    borrow::Cow,
    fmt::Display,
    process::{Command, ExitStatus, Output},
};

mod err_ctx;
mod err_ctx_flags;
mod err_kind;

use log::error;
use pareg::Pareg;

use crate::ext::Wrap;

pub use self::{err_ctx::*, err_ctx_flags::*, err_kind::*};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Result with the unified error type of uamp
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(Box<ErrCtx<ErrKind>>);

impl Error {
    pub fn new(kind: impl Into<ErrKind>) -> Self {
        Self(Box::new(ErrCtx::new(kind.into())))
    }

    pub fn invalid_operation() -> Self {
        Self::new(ErrKind::InvalidOperation)
    }

    pub fn msg(mut self, msg: impl Into<Cow<'static, str>>) -> Self {
        self.0.msg(msg);
        self
    }

    pub fn reason(mut self, reason: impl Into<Cow<'static, str>>) -> Self {
        self.0.reason(reason);
        self
    }

    pub fn hint(mut self, hint: impl Into<Cow<'static, str>>) -> Self {
        self.0.hint(hint);
        self
    }

    pub fn err<T>(self) -> Result<T> {
        Err(self)
    }

    pub fn kind(&self) -> &ErrKind {
        self.0.inner()
    }

    pub fn map_kind(mut self, f: impl FnOnce(ErrKind) -> ErrKind) -> Self {
        self.0.map_inner(f);
        self
    }

    pub fn map_pareg(self, args: &Pareg) -> Self {
        self.map_kind(|k| match k {
            ErrKind::Pareg(p) => args.map_err(p).into(),
            k => k,
        })
    }

    pub fn io(err: std::io::Error) -> Self {
        Self::new(ErrKind::Io(err))
    }

    pub fn no_stdin_pipe() -> Self {
        Self::new(ErrKind::NoStdinPipe)
    }

    pub fn warn(mut self) -> Self {
        self.0.warn();
        self
    }

    pub fn prepend(mut self, msg: impl Into<Cow<'static, str>>) -> Self {
        self.0.prepend(msg);
        self
    }

    pub fn inner_first(mut self) -> Self {
        self.0.inner_first(true);
        self
    }

    pub fn invalid_value() -> Self {
        Self::new(ErrKind::InvalidValue)
    }

    pub fn not_found() -> Self {
        Self::new(ErrKind::NotFound)
    }

    pub fn thread_panicked(e: Option<Box<dyn Any + Send + 'static>>) -> Self {
        let reason: Option<Cow<'static, str>> = e.and_then(|e| {
            e.downcast::<&'static str>()
                .map(|s| (*s).into())
                .or_else(|e| e.downcast::<String>().map(|s| (*s).into()))
                .ok()
        });

        if let Some(reason) = reason {
            Self::new(ErrKind::ThreadPanicked(reason))
        } else {
            Self::new(ErrKind::ThreadPanicked("Unknown message.".into()))
        }
    }

    pub fn unexpected() -> Self {
        Self::new(ErrKind::Unexpected(Wrap(Backtrace::capture())))
    }

    pub fn is_invalid_operation(&self) -> bool {
        matches!(self.kind(), ErrKind::InvalidOperation)
    }

    pub fn unsupported() -> Self {
        Self::new(ErrKind::Unsupported)
    }

    pub fn multiple(mut e: Vec<Error>) -> Result<()> {
        match e.len() {
            0 => Ok(()),
            1 => Err(e.pop().unwrap()),
            _ => Err(Error::new(ErrKind::Multiple(e))),
        }
    }

    pub fn http(code: u16, msg: impl Into<Cow<'static, str>>) -> Self {
        Self::new(ErrKind::Http(code, msg.into()))
    }

    pub fn other(err: impl Into<anyhow::Error>) -> Self {
        Self::new(ErrKind::Other(err.into()))
    }

    pub fn child_failed(
        code: ExitStatus,
        stderr: Option<impl Into<Cow<'static, str>>>,
    ) -> Self {
        Self::new(ErrKind::ChildFailed {
            code,
            stderr: stderr.map(|a| a.into()),
        })
    }

    pub fn child_output(res: &Output, cmd: Option<&Command>) -> Result<()> {
        if res.status.success() {
            Ok(())
        } else {
            let stderr = if res.stderr.is_empty() {
                None
            } else {
                Some(String::from_utf8_lossy(&res.stderr).to_string().into())
            };
            Self::child_status(res.status, stderr, cmd)
        }
    }

    pub fn child_status(
        status: ExitStatus,
        stderr: Option<Cow<'static, str>>,
        cmd: Option<&Command>,
    ) -> Result<()> {
        if status.success() {
            Ok(())
        } else {
            let err = Self::child_failed(status, stderr);
            if let Some(c) = cmd {
                err.hint(format!("When executing: {c:?}")).err()
            } else {
                err.err()
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.inner().source()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<T> for Error
where
    T: Into<ErrKind>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(value: tokio::task::JoinError) -> Self {
        if value.is_panic() {
            Self::thread_panicked(Some(value.into_panic()))
        } else {
            Self::unexpected().msg("Task was stopped.")
        }
    }
}

pub fn log_err<T, E: Display>(
    pf: impl Display,
    e: std::result::Result<T, E>,
) -> Option<T> {
    match e {
        Ok(v) => Some(v),
        Err(e) => {
            error!("{pf}: {e:-}");
            None
        }
    }
}
