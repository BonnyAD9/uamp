use std::{borrow::Cow, fmt::Display};

use pareg::ColorMode;
use termal::writemc;

#[derive(Debug)]
pub struct ErrCtx<T>
where
    T: Display,
{
    msg: Option<Cow<'static, str>>,
    inner: T,
    color: ColorMode,
}

impl<T> ErrCtx<T>
where
    T: Display,
{
    pub fn new(inner: T) -> Self {
        Self {
            msg: None,
            inner,
            color: ColorMode::default(),
        }
    }

    pub fn msg(mut self, msg: impl Into<Cow<'static, str>>) -> Self {
        self.msg = Some(msg.into());
        self
    }

    pub fn color(mut self, mode: ColorMode) -> Self {
        self.color = mode;
        self
    }

    pub fn no_color(self) -> Self {
        self.color(ColorMode::Never)
    }
}

impl<T> Display for ErrCtx<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = self.color.use_color();
        if f.sign_plus() {
            writemc!(f, color, "{'r}error:{'_} ")?;
        }

        if let Some(msg) = &self.msg {
            let msg = msg.trim_end_matches('.');
            write!(f, "{msg}: {}", self.inner)
        } else {
            write!(f, "{}", self.inner)
        }
    }
}
