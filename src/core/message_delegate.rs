use std::{fmt::Debug, sync::Arc};

use crate::env::AppCtrl;

use super::{Msg, UampApp};

pub trait MessageDelegate: Sync + Send + Debug {
    fn update(&self, app: &mut UampApp, ctrl: &mut AppCtrl) -> Option<Msg>;
}

pub struct FnDelegate<T>(T)
where
    T: Sync + Send + Fn(&mut UampApp, &mut AppCtrl) -> Option<Msg>;

impl<T> MessageDelegate for FnDelegate<T>
where
    T: Sync + Send + Fn(&mut UampApp, &mut AppCtrl) -> Option<Msg>,
{
    fn update(&self, app: &mut UampApp, ctrl: &mut AppCtrl) -> Option<Msg> {
        self.0(app, ctrl)
    }
}

impl<T> Debug for FnDelegate<T>
where
    T: Sync + Send + Fn(&mut UampApp, &mut AppCtrl) -> Option<Msg>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FnDelegate").finish()
    }
}

impl<T> From<T> for FnDelegate<T>
where
    T: Sync + Send + Fn(&mut UampApp, &mut AppCtrl) -> Option<Msg>,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl Msg {
    pub fn delegate<I, D>(d: I) -> Self
    where
        D: MessageDelegate + 'static,
        I: Into<D>,
    {
        Self::Delegate(Arc::new(d.into()))
    }
}
