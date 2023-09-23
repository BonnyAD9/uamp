use iced_core::Length::Fill;

use crate::{
    app::UampApp,
    gui::{
        ids::WB_SETTINGS_OTHER,
        wid::{space, Element},
    },
    wrap_box,
};

impl UampApp {
    pub(super) fn other(&self) -> Element {
        wrap_box![
            &self.gui.wb_states[WB_SETTINGS_OTHER],
            Self::gui_save(),
            self.gui_show_help(),
            self.gui_save_timeout(),
            self.gui_delete_logs_after(),
            self.gui_tick_length(),
            space(Fill, 20),
        ]
        .padding([0, 0, 0, 20])
        .spacing_y(5)
        .into()
    }
}
