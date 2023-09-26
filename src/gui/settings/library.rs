use iced_core::Length::Fill;

use crate::{
    app::UampApp,
    gui::{
        ids::WB_SETTINGS_LIBRARY,
        wid::{space, Element},
    },
    wrap_box,
};

impl UampApp {
    pub(super) fn library(&self) -> Element {
        wrap_box![
            &self.gui.wb_states[WB_SETTINGS_LIBRARY],
            Self::gui_search_for_new_songs(),
            self.gui_recursive_search(),
            self.gui_update_library_on_start(),
            self.gui_simple_sorting(),
            self.gui_search_paths(),
            self.gui_audio_extensions(),
            space(Fill, 20),
        ]
        .padding([0, 0, 0, 20])
        .spacing_y(5)
        .into()
    }
}
