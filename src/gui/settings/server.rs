use iced_core::Length::{Fill, Shrink};

use crate::{
    app::UampApp,
    col,
    gui::{
        ids::WB_SETTINGS_SERVER,
        wid::{space, Element},
    },
    wrap_box,
};

impl UampApp {
    pub(super) fn server(&self) -> Element {
        wrap_box![
            &self.gui.wb_states[WB_SETTINGS_SERVER],
            self.gui_enable_server(),
            col![self.gui_port(), self.gui_server_address()]
                .padding([0, 0, 0, 25])
                .height(Shrink),
            space(Fill, 20),
        ]
        .padding([0, 0, 0, 20])
        .spacing_y(5)
        .into()
    }
}
