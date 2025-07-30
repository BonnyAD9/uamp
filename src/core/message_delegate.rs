use std::fmt::Debug;

use crate::core::AppCtrl;

use super::{Msg, Result, UampApp};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Message that can do its own action.
pub trait MessageDelegate: Sync + Send + Debug {
    /// Action that this message does.
    fn update(
        self: Box<Self>,
        app: &mut UampApp,
        ctrl: &mut AppCtrl,
    ) -> Result<Vec<Msg>>;
}

/// Wrapper to implement [`MessageDelegate`] for closures.
pub struct FnDelegate<T>(T)
where
    T: Sync + Send + FnOnce(&mut UampApp, &mut AppCtrl) -> Result<Vec<Msg>>;

impl<T> MessageDelegate for FnDelegate<T>
where
    T: Sync + Send + FnOnce(&mut UampApp, &mut AppCtrl) -> Result<Vec<Msg>>,
{
    fn update(
        self: Box<Self>,
        app: &mut UampApp,
        ctrl: &mut AppCtrl,
    ) -> Result<Vec<Msg>> {
        self.0(app, ctrl)
    }
}

impl<T> Debug for FnDelegate<T>
where
    T: Sync + Send + FnOnce(&mut UampApp, &mut AppCtrl) -> Result<Vec<Msg>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FnDelegate").finish()
    }
}

impl<T> From<T> for FnDelegate<T>
where
    T: Sync + Send + FnOnce(&mut UampApp, &mut AppCtrl) -> Result<Vec<Msg>>,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl Msg {
    /// Creates delegate message.
    pub fn delegate<D>(d: D) -> Self
    where
        D: MessageDelegate + 'static,
    {
        Self::Delegate(Box::new(d))
    }

    pub fn fn_delegate<
        F: FnOnce(&mut UampApp, &mut AppCtrl) -> Result<Vec<Msg>>
            + Send
            + Sync
            + 'static,
    >(
        d: F,
    ) -> Self {
        Self::delegate(FnDelegate(d))
    }
}
