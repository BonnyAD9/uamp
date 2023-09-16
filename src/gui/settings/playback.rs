use iced_core::{alignment::Vertical, Length::{Shrink, Fill}};

use crate::{app::UampApp, gui::{wid::{Element, line_text, container, space}, ids::WB_SETTINGS_PLAYBACK, widgets::icons}, wrap_box, core::extensions::{duration_to_string, str_to_duration}, config::ConfMessage};

use super::{elements::{add_input, EmptyBehaviour, toggle}, SetMessage};

impl UampApp {
    pub(super) fn playback(&self) -> Element {
        wrap_box![
            &self.gui.wb_states[WB_SETTINGS_PLAYBACK],
            //=======================================<< Gapless playback toggle
            toggle(
                "Gapless playback",
                self.config.gapless(),
                ConfMessage::Gapless,
            ),

            //================================<< Fade play/pause duration input
            line_text(format!(
                "Fade play/pause: {}",
                duration_to_string(self.config.fade_play_pause().0, false)
            ))
            .height(30)
            .vertical_alignment(Vertical::Bottom)
            .padding([0, 0, 0, 10])
            .width(Shrink),
            container(add_input(
                "00:00.15",
                &self.gui.set_state.fade_play_pause_state,
                SetMessage::FadePlayPauseInput,
                |s| str_to_duration(s).is_some(),
                SetMessage::FadePlayPauseConfirm,
                icons::CHECK,
                EmptyBehaviour::Ignore,
            ))
            .padding([0, 0, 0, 25])
            .width(200)
            .height(Shrink),

            //=============================================<< Volume jump input
            line_text(format!(
                "Volume jump: {}",
                self.config.volume_jump() * 100.
            ))
            .height(30)
            .vertical_alignment(Vertical::Bottom)
            .padding([0, 0, 0, 10])
            .width(Shrink),
            container(add_input(
                "2.5",
                &self.gui.set_state.volume_jump_state,
                SetMessage::VolumeJumpInput,
                |s| s
                    .parse::<f32>()
                    .map(|v| (0.0..=1.).contains(&v))
                    .unwrap_or(false),
                SetMessage::VolumeJumpConfirm,
                icons::CHECK,
                EmptyBehaviour::Ignore,
            ))
            .padding([0, 0, 0, 25])
            .width(200)
            .height(Shrink),

            //===============================================<< Seek jump input
            line_text(format!(
                "Seek jump: {}",
                duration_to_string(self.config.seek_jump().0, false)
            ))
            .height(30)
            .vertical_alignment(Vertical::Bottom)
            .padding([0, 0, 0, 10])
            .width(Shrink),
            container(add_input(
                "00:10",
                &self.gui.set_state.seek_jump_state,
                SetMessage::SeekJumpInput,
                |s| str_to_duration(s).is_some(),
                SetMessage::SeekJumpConfirm,
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
