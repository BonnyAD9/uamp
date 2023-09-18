use iced_core::Length::{Fill, Shrink};
use itertools::Itertools;

use crate::{
    app::UampApp,
    col,
    config::ConfMessage,
    core::msg::Msg,
    gui::{
        ids::WB_SETTINGS_HOTKEYS,
        wid::{container, mouse_int, space, Element},
        widgets::icons,
        GuiMessage,
    },
    hotkeys::{Action, Hotkey},
    wrap_box,
};

use super::{
    elements::{add_input, delete_list, toggle, EmptyBehaviour},
    help, SetMessage,
};

impl UampApp {
    pub(super) fn hotkeys(&self) -> Element {
        wrap_box![
            &self.gui.wb_states[WB_SETTINGS_HOTKEYS],
            //=========================================<< Global hotkeys toggle
            mouse_int(toggle(
                "Global hotkeys",
                self.config.register_global_hotkeys(),
                ConfMessage::RegisterGlobalHotkeys
            ),)
            .on_mouse_enter(Msg::Gui(GuiMessage::Setings(
                SetMessage::ShowHelp(&help::ENABLE_GLOBAL_HOTKEYS)
            ))),
            //=====================================<< Global hotkeys list + add
            mouse_int(
                col![
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
                ]
                .spacing(5)
                .height(Shrink)
            )
            .on_mouse_enter(Msg::Gui(GuiMessage::Setings(
                SetMessage::ShowHelp(&help::GLOBAL_HOTKEY)
            ))),
            space(Fill, 20),
        ]
        .padding([0, 0, 0, 20])
        .spacing_y(5)
        .into()
    }
}
