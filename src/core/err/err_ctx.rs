use std::{borrow::Cow, fmt::Display};

use termal::{writemc, writemcln};

use super::ErrCtxFlags;

#[derive(Debug)]
pub struct ErrCtx<T>
where
    T: Display,
{
    inner: T,
    flags: ErrCtxFlags,
    msg: Option<Cow<'static, str>>,
    reason: Option<Cow<'static, str>>,
    hint: Option<Cow<'static, str>>,
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
        }
    }

    pub fn color(mut self, mode: ErrCtxFlags) -> Self {
        self.flags.set_color(mode);
        self
    }

    pub fn no_color(self) -> Self {
        self.color(ErrCtxFlags::COLOR_NEVER)
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

    pub fn show_err(mut self, v: bool) -> Self {
        self.flags.set(ErrCtxFlags::SHOW_ERR, v);
        self
    }
}

impl<T> Display for ErrCtx<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = self.flags.use_color();
        if self.flags.contains(ErrCtxFlags::SHOW_ERR) {
            writemc!(f, color, "{'r}error:{'_} ")?;
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

impl<T> From<T> for Box<ErrCtx<T>>
where
    T: Display,
{
    fn from(value: T) -> Self {
        Self::new(ErrCtx::new(value))
    }
}
