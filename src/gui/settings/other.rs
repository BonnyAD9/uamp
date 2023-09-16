use iced_core::{
    alignment::Vertical,
    Length::{Fill, Shrink},
};

use crate::{
    app::UampApp,
    core::{
        extensions::{duration_to_string, str_to_duration},
        msg::{ControlMsg, Msg},
    },
    gui::{
        elements::the_button,
        ids::WB_SETTINGS_OTHER,
        wid::{container, line_text, space, Element},
        widgets::icons,
    },
    wrap_box,
};

use super::{
    elements::{add_input, EmptyBehaviour},
    SetMessage,
};

impl UampApp {
    pub(super) fn other(&self) -> Element {
        wrap_box![
            &self.gui.wb_states[WB_SETTINGS_OTHER],
            //===================================================<< Save button
            the_button("Save").on_press(Msg::Control(ControlMsg::Save)),
            //============================================<< Save timeout input
            line_text(format!(
                "Save timeout: {}",
                self.config
                    .save_timeout()
                    .map(|t| duration_to_string(t.0, false))
                    .unwrap_or("never".to_owned())
            ))
            .height(30)
            .vertical_alignment(Vertical::Bottom)
            .padding([0, 0, 0, 10])
            .width(Shrink),
            container(add_input(
                "03:00",
                &self.gui.set_state.save_timeout_state,
                SetMessage::SaveTimeoutInput,
                |s| str_to_duration(s).is_some(),
                SetMessage::SaveTimeoutConfirm,
                icons::CHECK,
                EmptyBehaviour::Allow,
            ))
            .padding([0, 0, 0, 25])
            .width(200)
            .height(Shrink),
            //=======================================<< Delete logs after input
            line_text(format!(
                "Delete logs after: {}",
                duration_to_string(self.config.delete_logs_after().0, false)
            ))
            .height(30)
            .vertical_alignment(Vertical::Bottom)
            .padding([0, 0, 0, 10])
            .width(Shrink),
            container(add_input(
                "3d00:00",
                &self.gui.set_state.delete_logs_after_state,
                SetMessage::DeleteLogsAfterInput,
                |s| str_to_duration(s).is_some(),
                SetMessage::DeleteLogsAfterConfirm,
                icons::CHECK,
                EmptyBehaviour::Ignore,
            ))
            .padding([0, 0, 0, 25])
            .width(200)
            .height(Shrink),
            //=============================================<< Tick length input
            line_text(format!(
                "Tick length: {}",
                duration_to_string(self.config.tick_length().0, false)
            ))
            .height(30)
            .vertical_alignment(Vertical::Bottom)
            .padding([0, 0, 0, 10])
            .width(Shrink),
            container(add_input(
                "00:01",
                &self.gui.set_state.tick_length_state,
                SetMessage::TickLengthInput,
                |s| str_to_duration(s).is_some(),
                SetMessage::TickLengthConfirm,
                icons::CHECK,
                EmptyBehaviour::Ignore,
            ))
            .padding([0, 0, 0, 25])
            .width(200)
            .height(Shrink),
            space(Fill, 20),
        ]
        .padding([0, 0, 0, 20])
        .spacing_y(5)
        .into()
    }
}
