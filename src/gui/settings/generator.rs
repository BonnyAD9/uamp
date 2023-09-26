use place_macro::place;

use iced_core::{alignment::Vertical, Length::Shrink};

use itertools::Itertools;

use std::mem::replace;

use crate::{
    app::UampApp,
    col,
    config::ConfMessage,
    core::{
        extensions::{duration_to_string, str_to_duration},
        msg::{ComMsg, ControlMsg, Msg},
    },
    gui::{
        elements::the_button,
        ids::WB_SETTINGS_HELP,
        settings::{
            elements::{
                add_input, delete_list, title, toggle, EmptyBehaviour,
            },
            help::{self, SetHelp},
            Category,
        },
        wid::{container, line_text, mouse_int, MouseInteraction},
        widgets::icons,
        GuiMessage,
    },
};

macro_rules! generate_options {
    (
        $(
            $(
                button $button_vis:vis $button_name:ident: $button_text:literal
                    ? $button_help:ident => $button_msg:ident
            )?
            $(
                toggle $toggle_vis:vis $toggle_name:ident: $toggle_text:literal
                    ? $toggle_help:ident => $toggle_msg:ident
            )?
            $(
                list $list_vis:vis $list_name:ident
                    = |$list_item:ident| { $list_map:expr }:
                    { $list_text:expr } ($list_place:literal)
                    ? $list_help:ident => (
                        $list_remove_msg:ident,
                        $list_add_msg:ident:
                            |$list_var:ident| $list_parse:expr,
                        $list_empty:ident,
                        $list_width:literal,
                    )
            )?
            $(
                input $input_vis:vis $input_name:ident: ($input_self:ident)
                    { $input_text:expr } ($input_place:literal)
                    ? $input_help:ident => (
                        $input_msg:ident: |$input_var:ident| $input_parse:expr,
                        $input_empty:ident,
                        $input_width:literal,
                    )
            )?
        ),+
    ) => {
        place! {
            impl UampApp {
                $(
                    $(
                        #[inline(always)]
                        $button_vis fn __ident__(gui_ $button_name)<'a>()
                            -> MouseInteraction<'a>
                        {
                            mouse_int(the_button($button_text).on_press(
                                Msg::Control(ControlMsg::$button_msg)
                            ))
                            .on_mouse_enter(Msg::Gui(GuiMessage::Settings(
                                SetMessage::ShowHelp(&help::$button_help)
                            )))
                        }
                    )?
                    $(
                        #[inline(always)]
                        $toggle_vis fn __ident__(gui_ $toggle_name)(&self)
                            -> MouseInteraction
                        {
                            mouse_int(toggle(
                                $toggle_text,
                                self.config.$toggle_name(),
                                ConfMessage::$toggle_msg,
                            ))
                            .on_mouse_enter(Msg::Gui(GuiMessage::Settings(
                                SetMessage::ShowHelp(&help::$toggle_help)
                            )))
                        }
                    )?
                    $(
                        #[inline(always)]
                        $list_vis fn __ident__(gui_ $list_name)(&self)
                            -> MouseInteraction
                        {
                            mouse_int(
                                col![
                                    title($list_text),
                                    delete_list(
                                        self.config
                                            .$list_name()
                                            .iter()
                                            .map(|$list_item| {
                                                    $list_map
                                                }
                                                .into()
                                            ),
                                        ConfMessage::$list_remove_msg,
                                    ),
                                    container(add_input(
                                        $list_place,
                                        &self.gui
                                            .set_state
                                            .__ident__($list_name _state),
                                        SetMessage::__ident__(
                                                __ToCase__($list_name)
                                                Input
                                        ),
                                        |$list_var| { $list_parse }.is_some(),
                                        SetMessage::__ident__(
                                            __ToCase__($list_name)
                                            Confirm
                                        ),
                                        icons::ADD,
                                        EmptyBehaviour::$list_empty,
                                    ))
                                    .width($list_width)
                                    .height(Shrink)
                                    .padding([0, 0, 0, 25]),
                                ]
                                .width(Shrink)
                                .height(Shrink)
                                .spacing(5)
                            )
                            .on_mouse_enter(Msg::Gui(GuiMessage::Settings(
                                SetMessage::ShowHelp(&help::$list_help)
                            )))
                        }
                    )?
                    $(
                        #[inline(always)]
                        $input_vis fn __ident__(gui_ $input_name)(&$input_self)
                            -> MouseInteraction
                        {
                            mouse_int(
                                col![
                                    line_text($input_text)
                                        .height(30)
                                        .vertical_alignment(Vertical::Bottom)
                                        .padding([0, 0, 0, 10])
                                        .width(Shrink),
                                    container(add_input(
                                        $input_place,
                                        &$input_self
                                            .gui
                                            .set_state
                                            .__ident__($input_name _state),
                                        SetMessage::__ident__(
                                            __ToCase__($input_name)
                                            Input
                                        ),
                                        |$input_var| {
                                            $input_parse
                                        }
                                        .is_some(),
                                        SetMessage::__ident__(
                                            __ToCase__($input_name)
                                            Confirm
                                        ),
                                        icons::CHECK,
                                        EmptyBehaviour::$input_empty,
                                    ))
                                    .padding([0, 0, 0, 25])
                                    .width($input_width)
                                    .height(Shrink)
                                ]
                                .spacing(5)
                                .width(Shrink)
                                .height(Shrink)
                            )
                            .on_mouse_enter(Msg::Gui(GuiMessage::Settings(
                                SetMessage::ShowHelp(&help::$input_help)
                            )))
                        }
                    )?
                )+
            }

            #[derive(Default)]
            pub struct SetState {
                pub(super) help: Option<&'static SetHelp>,
                pub(super) category: Category,
                pub(super) hotkey_state: String,
                $(
                    $(__ident__($list_name _state): String,)?
                    $(__ident__($input_name _state): String,)?
                )+
            }

            #[derive(Clone, Debug)]
            pub enum SetMessage {
                SetCategory(Category),
                ShowHelp(&'static SetHelp),
                HotkeyInput(String),
                HotkeyConfirm,
                $(
                    $(__ident__(__ToCase__($list_name) Input)(String),)?
                    $(__ident__(__ToCase__($list_name) Confirm),)?
                    $(__ident__(__ToCase__($input_name) Input)(String),)?
                    $(__ident__(__ToCase__($input_name) Confirm),)?
                )+
            }

            impl UampApp {
                pub(super) fn settings_event_inner(&mut self, msg: SetMessage)
                    -> ComMsg
                {
                    match msg {
                        SetMessage::SetCategory(c) => {
                            self.gui.set_state.category = c;
                            self.gui.set_state.help = None;
                        }
                        SetMessage::ShowHelp(h) => {
                            if let Some(oh) = self.gui.set_state.help {
                                if !std::ptr::eq(oh, h) {
                                    self.gui.wb_states[WB_SETTINGS_HELP]
                                        .get_mut()
                                        .scroll_to_top()
                                }
                            }
                            self.gui.set_state.help = Some(h);
                        }
                        SetMessage::HotkeyInput(s) => {
                            self.gui.set_state.hotkey_state = s
                        }
                        SetMessage::HotkeyConfirm => {
                            let s = replace(
                                &mut self.gui.set_state.hotkey_state,
                                String::new(),
                            );
                            let s = s
                                .split(':')
                                .map(|s| s.trim())
                                .collect_vec();
                            return ComMsg::Msg(Msg::Config(
                                ConfMessage::AddGlobalHotkey(
                                    s[0].to_string(),
                                    s[1].to_string(),
                                ),
                            ));
                        }
                        $(
                            $(
                                SetMessage::__ident__(
                                    __ToCase__($list_name)
                                    Input
                                )(s) => {
                                    self.gui
                                        .set_state
                                        .__ident__($list_name _state)
                                        = s;
                                }
                            )?
                            $(
                                SetMessage::__ident__(
                                    __ToCase__($input_name)
                                    Input
                                )(s) => {
                                    self.gui
                                        .set_state
                                        .__ident__($input_name _state)
                                        = s;
                                }
                            )?
                            $(
                                SetMessage::__ident__(
                                    __ToCase__($list_name)
                                    Confirm
                                ) => {
                                    let $list_var = replace(
                                        &mut self
                                            .gui
                                            .set_state
                                            .__ident__($list_name _state),
                                        String::new(),
                                    );
                                    if let Some(s) = $list_parse {
                                        return ComMsg::Msg(Msg::Config(
                                            ConfMessage::$list_add_msg(
                                                s.into()
                                            )
                                        ))
                                    }
                                }
                            )?
                            $(
                                SetMessage::__ident__(
                                    __ToCase__($input_name)
                                    Confirm
                                ) => {
                                    let $input_var = replace(
                                        &mut self
                                            .gui
                                            .set_state
                                            .__ident__($input_name _state),
                                        String::new(),
                                    );
                                    if let Some(s) = $input_parse {
                                        return ComMsg::Msg(Msg::Config(
                                            ConfMessage::$input_msg(s.into())
                                        ))
                                    }
                                }
                            )?
                        )+
                    }

                    ComMsg::none()
                }
            }
        }
    };
}

generate_options! {
    //===============================================================<< LIBRARY
    button pub(super) search_for_new_songs: "Search for new songs"
        ? SEARCH_FOR_NEW_SONGS => LoadNewSongs,

    toggle pub(super) recursive_search: "Recursive search for new songs"
        ? RECURSIVE_SEARCH_FOR_NEW_SONGS => RecursiveSearch,

    toggle pub(super) update_library_on_start: "Update library on start"
        ? UPDATE_LIBRARY_ON_START => UpdateLibraryOnStart,

    toggle pub(super) simple_sorting: "Simple sorting"
        ? SIMPLE_SORTING => SimpleSorting,

    list pub(super) search_paths = |p| { p.to_string_lossy() }:
        { "Library search paths" } ("path")
        ? LIBRARY_SEARCH_PATHS => (
            RemoveSearchPath,
            AddSearchPath: |s| Some(s),
            Ignore,
            400,
        ),

    list pub(super) audio_extensions = |s| { s }:
        { "Song extensions" } ("extension")
        ? SONG_EXTENSIONS => (
            RemoveAudioExtension,
            AddAudioExtension: |s| Some(s),
            Ignore,
            200,
        ),

    //==============================================================<< PLAYBACK
    toggle pub(super) gapless: "Gapless playback"
        ? GAPLESS_PLAYBACK => Gapless,

    toggle pub(super) shuffle_current: "Shuffle now playing"
        ? SHUFFLE_NOW_PLAYING => ShuffleCurrent,

    toggle pub(super) show_remaining_time: "Show remaining time"
        ? SHOW_REMAINING_TIME => ShowRemainingTime,

    toggle pub(super) play_on_start: "Play on start"
        ? PLAY_ON_START => PlayOnStart,

    input pub(super) fade_play_pause: (self) {
        format!(
            "Fade play/pause: {}",
            duration_to_string(self.config.fade_play_pause().0, false)
        )
    } ("00:00.15") ? FADE_PLAY_PAUSE => (
        FadePlayPause: |s| str_to_duration(s.as_ref()),
        Ignore,
        200,
    ),

    input pub(super) volume_jump: (self) {
        format!(
            "Fade play/pause: {}",
            self.config.volume_jump(),
        )
    } ("2.5") ? VOLUME_JUMP => (
        VolumeJump: |s| {
            if let Ok(v) = s.parse::<f32>() {
                if (0.0..100.).contains(&v) {
                    Some(v / 100.)
                } else {
                    None
                }
            } else {
                None
            }
        },
        Ignore,
        200,
    ),

    input pub(super) seek_jump: (self) {
        format!(
            "Seek jump: {}",
            duration_to_string(self.config.seek_jump().0, false)
        )
    } ("00:10") ? SEEK_JUMP => (
        SeekJump: |s| str_to_duration(s.as_ref()),
        Ignore,
        200,
    ),

    input pub(super) previous_timeout: (self) {
        format!(
            "Previous timeout: {}",
            self.config.previous_timeout()
                .map(|d| duration_to_string(d.0, false))
                .unwrap_or("disabled".to_owned())
        )
    } ("00:10") ? PREVIOUS_TIMEOUT => (
        PreviousTimeout: |s| {
            if s.is_empty() {
                Some(None)
            } else {
                str_to_duration(s.as_ref()).map(|d| Some(d))
            }
        },
        Allow,
        200,
    ),

    //===============================================================<< HOTKEYS

    //================================================================<< SERVER
    toggle pub(super) enable_server: "Enable server"
        ? ENABLE_SERVER_FOR_CLI => EnableServer,

    input pub(super) port: (self) {
        format!(
            "Server port: {}",
            self.config.port(),
        )
    } ("8267 / 33284") ? SERVER_PORT => (
        Port: |s| s.parse::<u16>().ok(),
        Ignore,
        200,
    ),

    input pub(super) server_address: (self) {
        format!(
            "Server address: {}",
            self.config.server_address(),
        )
    } ("127.0.0.1") ? SERVER_ADDRESS => (
        ServerAddress: |s| Some(s),
        Ignore,
        400,
    ),

    //=================================================================<< OTHER
    button pub(super) save: "Save" ? SAVE_BUTTON => Save,

    toggle pub(super) show_help: "Show help" ? SHOW_HELP => ShowHelp,

    input pub(super) save_timeout: (self) {
        format!(
            "Save timeout: {}",
            self.config.save_timeout()
                .map(|d| duration_to_string(d.0, false))
                .unwrap_or("disabled".to_owned())
        )
    } ("01:00") ? SAVE_TIMEOUT => (
        SaveTimeout: |s| {
            if s.is_empty() {
                Some(None)
            } else {
                str_to_duration(s.as_ref()).map(|d| Some(d))
            }
        },
        Allow,
        200,
    ),

    input pub(super) delete_logs_after: (self) {
        format!(
            "Delete logs after: {}",
            duration_to_string(self.config.delete_logs_after().0, false)
        )
    } ("3d00:00") ? DELETE_LOGS_AFTER => (
        DeleteLogsAfter: |s| str_to_duration(s.as_ref()),
        Ignore,
        200,
    ),

    input pub(super) tick_length: (self) {
        format!(
            "Tick length: {}",
            duration_to_string(self.config.tick_length().0, false)
        )
    } ("00:01") ? TICK_LENGTH => (
        TickLength: |s| str_to_duration(s.as_ref()),
        Ignore,
        200,
    ),
}
