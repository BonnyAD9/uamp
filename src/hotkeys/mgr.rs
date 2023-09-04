use std::{collections::HashMap, str::FromStr, sync::Arc};

use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::core::msg::Msg;

use super::{action::Action, err, hotkey::Hotkey};

#[derive(Clone)]
pub struct HotkeyMgr {
    unparsed: HashMap<String, String>,
    parsed: HashMap<Hotkey, Action>,
}

impl HotkeyMgr {
    pub fn new() -> Self {
        Self {
            unparsed: HashMap::new(),
            parsed: HashMap::new(),
        }
    }

    fn parse(&mut self) {
        self.parsed.clear();
        for (h, ha) in self.unparsed.iter() {
            let h = match Hotkey::from_str(h) {
                Ok(r) => r,
                Err(e) => {
                    error!("Failed to parse hotkey: {e}");
                    continue;
                }
            };
            let ha = match Action::from_str(ha) {
                Ok(r) => r,
                Err(e) => {
                    error!("Failed to parse hotkey action: {e}");
                    continue;
                }
            };

            // If the hotkey is present, combine them
            if let Some(a) = self.parsed.get_mut(&h) {
                a.join(ha);
            } else {
                self.parsed.insert(h, ha);
            }
        }
    }

    pub fn register(
        &mut self,
        sender: Arc<UnboundedSender<Msg>>,
    ) -> Result<GlobalHotKeyManager, err::Error> {
        self.parse();

        let res = GlobalHotKeyManager::new()?;

        let mut hotkeys = HashMap::new();

        for (h, a) in self.parsed.iter() {
            let h = h.as_hot_key();
            hotkeys.insert(h.id(), a.clone());
            res.register(h)?;
        }

        GlobalHotKeyEvent::set_event_handler(Some(
            move |e: GlobalHotKeyEvent| {
                let a = match hotkeys.get(&e.id) {
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

        Ok(res)
    }

    /// Adds hotkey
    pub fn add_hotkey<S>(&mut self, hotkey: S, action: S)
    where
        S: Into<String>,
    {
        self.unparsed.insert(hotkey.into(), action.into());
    }
}

impl Serialize for HotkeyMgr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.unparsed.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for HotkeyMgr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        HashMap::deserialize(deserializer).map(|r| Self {
            unparsed: r,
            parsed: HashMap::new(),
        })
    }
}
