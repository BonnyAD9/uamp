use iced_core::Length::{Fill, Shrink};
use itertools::Itertools;

use crate::{
    app::UampApp,
    config::ConfMessage,
    gui::{
        ids::WB_SETTINGS_HOTKEYS,
        wid::{container, space, Element},
        widgets::icons,
    },
    hotkeys::{Action, Hotkey},
    wrap_box,
};

use super::{
    elements::{add_input, delete_list, toggle, EmptyBehaviour},
    SetMessage,
};

impl UampApp {
    pub(super) fn hotkeys(&self) -> Element {
        wrap_box![
            &self.gui.wb_states[WB_SETTINGS_HOTKEYS],
            //=========================================<< Global hotkeys toggle
            toggle(
                "Global hotkeys",
                self.config.register_global_hotkeys(),
                ConfMessage::RegisterGlobalHotkeys
            ),
            //=====================================<< Global hotkeys list + add
            delete_list(
                self.config
                    .global_hotkeys()
                    .iter()
                    .map(|(h, a)| format!("{h}: {a}").into()),
                ConfMessage::RemoveGlobalHotkey
            ),
            container(add_input(
                "hotkey: action",
                &self.gui.set_state.hotkey_state,
                SetMessage::HotkeyInput,
                |s| {
                    let s = s.split(':').collect_vec();
                    s.len() == 2
                        && s[0].parse::<Hotkey>().is_ok()
                        && s[1].parse::<Action>().is_ok()
                },
                SetMessage::HotkeyConfirm,
                icons::ADD,
                EmptyBehaviour::Ignore,
            ))
            .width(400)
            .height(Shrink)
            .padding([0, 0, 0, 25]),
            space(Fill, 20),
        ]
        .padding([0, 0, 0, 20])
        .spacing_y(5)
        .into()
    }
}
