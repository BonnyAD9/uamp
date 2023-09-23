use iced_core::Length::Fill;

use crate::{
    app::UampApp,
    gui::{
        ids::WB_SETTINGS_PLAYBACK,
        wid::{space, Element},
    },
    wrap_box,
};

impl UampApp {
    pub(super) fn playback(&self) -> Element {
        wrap_box![
            &self.gui.wb_states[WB_SETTINGS_PLAYBACK],
            self.gui_gapless(),
            self.gui_shuffle_current(),
            self.gui_show_remaining_time(),
            self.gui_fade_play_pause(),
            self.gui_volume_jump(),
            self.gui_seek_jump(),
            self.gui_previous_timeout(),
            space(Fill, 20),
        ]
        .padding([0, 0, 0, 20])
        .spacing_y(5)
        .into()
    }
}
