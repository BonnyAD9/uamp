use std::{borrow::Cow, fmt::Display};

use serde::{Deserialize, Serialize};
use termal::{writemc, writemcln};

use super::ErrCtxFlags;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrCtx<T>
where
    T: Display,
{
    inner: T,
    flags: ErrCtxFlags,
    msg: Option<Cow<'static, str>>,
    reason: Option<Cow<'static, str>>,
    hint: Option<Cow<'static, str>>,
    prepend: Vec<Cow<'static, str>>,
}

impl<T> ErrCtx<T>
where
    T: Display,
{
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            flags: ErrCtxFlags::default(),
            msg: None,
            reason: None,
            hint: None,
            prepend: vec![],
        }
    }

    pub fn color(mut self, mode: ErrCtxFlags) -> Self {
        self.flags.set_color(mode);
        self
    }

    pub fn no_color(self) -> Self {
        self.color(ErrCtxFlags::COLOR_NEVER)
    }

    pub fn severity(mut self, severity: ErrCtxFlags) -> Self {
        self.flags.set_severity(severity);
        self
    }

    pub fn warn(self) -> Self {
        self.severity(ErrCtxFlags::SEVERITY_WARN)
    }

    pub fn inner_first(mut self, v: bool) -> Self {
        self.flags.set(ErrCtxFlags::INNER_FIRST, v);
        self
    }

    pub fn msg(mut self, msg: impl Into<Cow<'static, str>>) -> Self {
        self.msg = Some(msg.into());
        self
    }

    pub fn reason(mut self, reason: impl Into<Cow<'static, str>>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    pub fn hint(mut self, hint: impl Into<Cow<'static, str>>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn prepend(mut self, msg: impl Into<Cow<'static, str>>) -> Self {
        self.prepend.push(msg.into());
        self
    }

    pub fn show_err(mut self, v: bool) -> Self {
        self.flags.set(ErrCtxFlags::SHOW_ERR, v);
        self
    }

    pub fn clone_universal(&self) -> ErrCtx<String> {
        ErrCtx {
            inner: self.inner.to_string(),
            flags: self.flags,
            msg: self.msg.clone(),
            reason: self.reason.clone(),
            hint: self.hint.clone(),
            prepend: self.prepend.clone(),
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T> Display for ErrCtx<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = self.flags.use_color();
        if self.flags.contains(ErrCtxFlags::SHOW_ERR) {
            match self.flags & ErrCtxFlags::SEVERITY {
                ErrCtxFlags::SEVERITY_ERROR => {
                    writemc!(f, color, "{'r}error:{'_} ")?
                }
                ErrCtxFlags::SEVERITY_WARN => {
                    writemc!(f, color, "{'m}warn:{'_} ")?
                }
                _ => writemc!(f, color, "{'r}info:{'_} ")?,
            }
        }

        for p in self.prepend.iter().map(|a| a.as_ref()).rev() {
            let msg = p.trim_end_matches('.');
            write!(f, "{msg}: ")?;
        }

        if let Some(msg) = &self.msg {
            if self.flags.contains(ErrCtxFlags::INNER_FIRST) {
                let inner = self.inner.to_string();
                let inner = inner.trim_end_matches('.');
                writeln!(f, "{inner}: {msg}")?;
            } else {
                let msg = msg.trim_end_matches('.');
                writeln!(f, "{msg}: {}", self.inner)?;
            }
        } else {
            writeln!(f, "{}", self.inner)?;
        }

        if let Some(reason) = &self.reason {
            writemcln!(f, color, "{'y}reason:{'_} {reason}")?;
        }

        if let Some(hint) = &self.hint {
            writemcln!(f, color, "{'c}hint:{'_} {hint}")?;
        }

        Ok(())
    }
}

impl<T> From<T> for ErrCtx<T>
where
    T: Display,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> From<T> for Box<ErrCtx<T>>
where
    T: Display,
{
    fn from(value: T) -> Self {
        Self::new(ErrCtx::new(value))
    }
}
