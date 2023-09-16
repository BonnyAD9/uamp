use iced_core::{
    alignment::Vertical,
    Length::{Fill, Shrink},
};

use crate::{
    app::UampApp,
    col,
    config::ConfMessage,
    gui::{
        ids::WB_SETTINGS_SERVER,
        wid::{container, line_text, space, Element},
        widgets::icons,
    },
    wrap_box,
};

use super::{
    elements::{add_input, toggle, EmptyBehaviour},
    SetMessage,
};

impl UampApp {
    pub(super) fn server(&self) -> Element {
        wrap_box![
            &self.gui.wb_states[WB_SETTINGS_SERVER],
            //==================================<< Enable server for CLI toggle
            toggle(
                "Enable server for CLI",
                self.config.enable_server(),
                ConfMessage::EnableServer,
            ),
            col![
                //=========================================<< Server port input
                line_text(format!("Server port: {}", self.config.port()))
                    .height(30)
                    .vertical_alignment(Vertical::Bottom)
                    .padding([0, 0, 0, 10])
                    .width(Shrink),
                container(add_input(
                    "8267 / 33284",
                    &self.gui.set_state.port_state,
                    SetMessage::PortInput,
                    |s| s.parse::<u32>().is_ok(),
                    SetMessage::PortConfirm,
                    icons::CHECK,
                    EmptyBehaviour::Ignore,
                ))
                .padding([0, 0, 0, 25])
                .width(200)
                .height(Shrink),
                //======================================<< Server address input
                line_text(format!(
                    "Server address: {}",
                    self.config.server_address()
                ))
                .height(30)
                .vertical_alignment(Vertical::Bottom)
                .padding([0, 0, 0, 10])
                .width(Shrink),
                container(add_input(
                    "127.0.0.1",
                    &self.gui.set_state.server_address_state,
                    SetMessage::ServerAddressInput,
                    |_| true,
                    SetMessage::ServerAddressConfirm,
                    icons::CHECK,
                    EmptyBehaviour::Ignore,
                ))
                .padding([0, 0, 0, 25])
                .width(200)
                .height(Shrink),
            ]
            .padding([0, 0, 0, 25])
            .height(Shrink),
            space(Fill, 20),
        ]
        .padding([0, 0, 0, 20])
        .spacing_y(5)
        .into()
    }
}
