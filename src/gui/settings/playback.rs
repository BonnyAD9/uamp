use iced_core::{
    alignment::Vertical,
    Length::{Fill, Shrink},
};

use crate::{
    app::UampApp,
    col,
    config::ConfMessage,
    core::{
        extensions::{duration_to_string, str_to_duration},
        msg::Msg,
    },
    gui::{
        ids::WB_SETTINGS_PLAYBACK,
        wid::{container, line_text, mouse_int, space, Element},
        widgets::icons,
        GuiMessage,
    },
    wrap_box,
};

use super::{
    elements::{add_input, toggle, EmptyBehaviour},
    help, SetMessage,
};

impl UampApp {
    pub(super) fn playback(&self) -> Element {
        wrap_box![
            &self.gui.wb_states[WB_SETTINGS_PLAYBACK],
            //=======================================<< Gapless playback toggle
            mouse_int(toggle(
                "Gapless playback",
                self.config.gapless(),
                ConfMessage::Gapless,
            ),)
            .on_mouse_enter(Msg::Gui(GuiMessage::Setings(
                SetMessage::ShowHelp(&help::GAPLESS_PLAYBACK)
            ))),
            //===========================================<< Shuffle now playing
            mouse_int(toggle(
                "Shuffle now playing",
                self.config.shuffle_current(),
                ConfMessage::ShuffleCurrent,
            ),)
            .on_mouse_enter(Msg::Gui(GuiMessage::Setings(
                SetMessage::ShowHelp(&help::SHUFFLE_NOW_PLAYING)
            ))),
            //===========================================<< Show remaining time
            mouse_int(toggle(
                "Show remaining time",
                self.config.show_remaining_time(),
                ConfMessage::ShowRemainingTime,
            ),)
            .on_mouse_enter(Msg::Gui(GuiMessage::Setings(
                SetMessage::ShowHelp(&help::SHOW_REMAINING_TIME)
            ))),
            //================================<< Fade play/pause duration input
            mouse_int(
                col![
                    line_text(format!(
                        "Fade play/pause: {}",
                        duration_to_string(
                            self.config.fade_play_pause().0,
                            false
                        )
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
                ]
                .spacing(5)
                .height(Shrink)
            )
            .on_mouse_enter(Msg::Gui(GuiMessage::Setings(
                SetMessage::ShowHelp(&help::FADE_PLAY_PAUSE)
            ))),
            //=============================================<< Volume jump input
            mouse_int(
                col![
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
                            .map(|v| (0.0..=100.).contains(&v))
                            .unwrap_or(false),
                        SetMessage::VolumeJumpConfirm,
                        icons::CHECK,
                        EmptyBehaviour::Ignore,
                    ))
                    .padding([0, 0, 0, 25])
                    .width(200)
                    .height(Shrink),
                ]
                .spacing(5)
                .height(Shrink)
            )
            .on_mouse_enter(Msg::Gui(GuiMessage::Setings(
                SetMessage::ShowHelp(&help::VOLUME_JUMP)
            ))),
            //===============================================<< Seek jump input
            mouse_int(
                col![
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
                ]
                .spacing(5)
                .height(Shrink)
            )
            .on_mouse_enter(Msg::Gui(GuiMessage::Setings(
                SetMessage::ShowHelp(&help::SEEK_JUMP)
            ))),
            //========================================<< Previous timeout input
            mouse_int(
                col![
                    line_text(format!(
                        "Previous timeout: {}",
                        self.config
                            .previous_timeout()
                            .map(|t| duration_to_string(t.0, false))
                            .unwrap_or("disabled".to_owned())
                    ))
                    .height(30)
                    .vertical_alignment(Vertical::Bottom)
                    .padding([0, 0, 0, 10])
                    .width(Shrink),
                    container(add_input(
                        "",
                        &self.gui.set_state.previous_timeout_state,
                        SetMessage::PreviousTimeoutInput,
                        |s| str_to_duration(s).is_some(),
                        SetMessage::PreviousTimeoutConfirm,
                        icons::CHECK,
                        EmptyBehaviour::Allow,
                    ))
                    .padding([0, 0, 0, 25])
                    .width(200)
                    .height(Shrink),
                ]
                .spacing(5)
                .height(Shrink)
            )
            .on_mouse_enter(Msg::Gui(GuiMessage::Setings(
                SetMessage::ShowHelp(&help::PREVIOUS_TIMEOUT)
            ))),
            space(Fill, 20),
        ]
        .padding([0, 0, 0, 20])
        .spacing_y(5)
        .into()
    }
}
