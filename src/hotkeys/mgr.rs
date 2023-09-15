use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex},
};

use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use log::error;
use tokio::sync::mpsc::UnboundedSender;

use crate::core::{err::Result, msg::Msg, Error};

use super::{action::Action, hotkey::Hotkey};

/// Parses hotkeys and registers them
pub struct HotkeyMgr {
    mapping: Option<Arc<Mutex<HashMap<u32, Action>>>>,
    inner: Option<GlobalHotKeyManager>,
}

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

impl HotkeyMgr {
    /// Creates new [`HotkeyMgr`]
    pub fn new() -> Self {
        Self {
            mapping: None,
            inner: None,
        }
    }

    /// Parses and registers the hotkeys
    pub fn init<I>(
        &mut self,
        sender: Arc<UnboundedSender<Msg>>,
        hotkeys: I,
    ) -> Result<()>
    where
        I: Iterator<Item = (Hotkey, Action)>,
    {
        // Drop previous
        self.inner = None;
        self.mapping = None;

        let res = GlobalHotKeyManager::new()?;

        let mut map: HashMap<u32, Action> = HashMap::new();

        for (h, a) in hotkeys {
            let h = h.as_hot_key();
            if let Some(oa) = map.get_mut(&h.id()) {
                oa.join(a);
                continue;
            }
            map.insert(h.id(), a);
            if let Err(e) = res.register(h) {
                error!("Failed to register hotkey: {e}")
            }
        }

        let map = Arc::new(Mutex::new(map));
        self.mapping = Some(map.clone());

        GlobalHotKeyEvent::set_event_handler(Some(
            move |e: GlobalHotKeyEvent| {
                let map = match map.lock() {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Failed to lock shortcut map: {e}");
                        return;
                    }
                };

                let a = match map.get(&e.id) {
                    Some(a) => a,
                    None => return,
                };

                for m in &a.controls {
                    if let Err(e) = sender.send(Msg::Control(*m)) {
                        error!("Failed to send hotkey message: {e}")
                    }
                }
            },
        ));

        self.inner = Some(res);

        Ok(())
    }

    /// Adds hotkey
    pub fn add_hotkey(&self, hotkey: Hotkey, action: Action) -> Result<()> {
        let (mgr, map) = if let (Some(i), Some(m)) =
            (&self.inner, &self.mapping)
        {
            (i, m)
        } else {
            return Err(Error::InvalidOperation("Cannot register hotkey: the hotkey manager is not initialized"));
        };

        let hotkey = hotkey.as_hot_key();

        {
            let mut map = map.lock()?;
            if let Some(a) = map.get_mut(&hotkey.id()) {
                a.join(action);
                return Ok(());
            }
            map.insert(hotkey.id(), action);
        }

        //map.lock()?.insert(hotkey.id(), action);

        mgr.register(hotkey)?;

        Ok(())
    }

    pub fn remove_hotkey(
        &self,
        hotkey: &Hotkey,
        action: &Action,
    ) -> Result<()> {
        let (mgr, map) = if let (Some(i), Some(m)) =
            (&self.inner, &self.mapping)
        {
            (i, m)
        } else {
            return Err(Error::InvalidOperation("Cannot register hotkey: the hotkey manager is not initialized"));
        };

        let hotkey = hotkey.as_hot_key();

        let mut map = map.lock()?;
        if let Some(mut a) = map.remove(&hotkey.id()) {
            a.strip(&action);
            if a.controls.is_empty() {
                mgr.unregister(hotkey)?;
            }
        }

        Ok(())
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//
