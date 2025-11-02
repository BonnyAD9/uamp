use std::{borrow::Cow, fmt::Display};

use serde::{Deserialize, Serialize};
use termal::{writemc, writemcln};

use super::ErrCtxFlags;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrCtx<T>
where
    T: Display,
{
    inner: Option<T>,
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
            inner: Some(inner),
            flags: ErrCtxFlags::default(),
            msg: None,
            reason: None,
            hint: None,
            prepend: vec![],
        }
    }

    pub fn color(&mut self, mode: ErrCtxFlags) {
        self.flags.set_color(mode);
    }

    pub fn no_color(&mut self) {
        self.color(ErrCtxFlags::COLOR_NEVER);
    }

    pub fn severity(&mut self, severity: ErrCtxFlags) {
        self.flags.set_severity(severity);
    }

    pub fn warn(&mut self) {
        self.severity(ErrCtxFlags::SEVERITY_WARN);
    }

    pub fn inner_first(&mut self, v: bool) {
        self.flags.set(ErrCtxFlags::INNER_FIRST, v);
    }

    pub fn msg(&mut self, msg: impl Into<Cow<'static, str>>) {
        self.msg = Some(msg.into());
    }

    pub fn reason(&mut self, reason: impl Into<Cow<'static, str>>) {
        self.reason = Some(reason.into());
    }

    pub fn hint(&mut self, hint: impl Into<Cow<'static, str>>) {
        self.hint = Some(hint.into());
    }

    pub fn prepend(&mut self, msg: impl Into<Cow<'static, str>>) {
        self.prepend.push(msg.into());
    }

    pub fn show_err(&mut self, v: bool) {
        self.flags.set(ErrCtxFlags::SHOW_ERR, v);
    }

    pub fn clone_universal(&self) -> ErrCtx<String> {
        ErrCtx {
            inner: Some(self.inner().to_string()),
            flags: self.flags,
            msg: self.msg.clone(),
            reason: self.reason.clone(),
            hint: self.hint.clone(),
            prepend: self.prepend.clone(),
        }
    }

    pub fn inner(&self) -> &T {
        self.inner.as_ref().unwrap()
    }

    pub fn map_inner(&mut self, f: impl FnOnce(T) -> T) {
        self.inner = Some(f(std::mem::take(&mut self.inner).unwrap()));
    }
}

impl<T> Display for ErrCtx<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flags = self.flags;
        if f.sign_minus() {
            flags.set_color(ErrCtxFlags::COLOR_NEVER);
        }

        let color = flags.use_color();
        if self.flags.contains(ErrCtxFlags::SHOW_ERR) {
            match flags & ErrCtxFlags::SEVERITY {
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

        let fmt_inner = || {
            if color {
                format!("{:+}", self.inner())
            } else {
                format!("{:-}", self.inner())
            }
        };

        if let Some(msg) = &self.msg {
            if flags.contains(ErrCtxFlags::INNER_FIRST) {
                let inner = fmt_inner();
                let inner = inner.trim_end_matches('.');
                writeln!(f, "{inner}: {msg}")?;
            } else {
                let msg = msg.trim_end_matches('.');
                writeln!(f, "{msg}: {}", fmt_inner())?;
            }
        } else {
            writeln!(f, "{}", self.inner())?;
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
