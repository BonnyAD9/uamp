use std::{borrow::Cow, mem::replace};

use iced_core::{
    alignment::Vertical,
    font::Weight,
    svg, Font,
    Length::{Fill, Shrink},
};
use itertools::Itertools;
use log::error;

use crate::{
    app::UampApp,
    col,
    config::ConfMessage,
    core::{
        extensions::{duration_to_string, str_to_duration},
        msg::{ComMsg, ControlMsg, Msg},
    },
    gui::ids::WB_SETTINGS,
    hotkeys::{Action, Hotkey},
    row, wrap_box,
};

use super::{
    elements::the_button,
    theme::{Container, SvgButton, Text, TextInput},
    wid::{
        self, column, container, cursor_grad, line_text, space, svg_button,
        switch, text, text_input, Element,
    },
    widgets::icons,
    GuiMessage,
};

#[derive(Default)]
pub struct SetState {
    extension_state: String,
    search_path_state: String,
    volume_jump_state: String,
    save_timeout_state: String,
    seek_jump_state: String,
    delete_logs_after_state: String,
    hotkey_state: String,
    tick_length_state: String,
    port_state: String,
    server_address_state: String,
}

#[derive(Clone, Debug)]
pub enum SetMessage {
    ExtensionInput(String),
    ExtensionConfirm,
    SearchPathInput(String),
    SearchPathConfirm,
    VolumeJumpInput(String),
    VolumeJumpConfirm,
    SaveTimeoutInput(String),
    SaveTimeoutConfirm,
    SeekJumpInput(String),
    SeekJumpConfirm,
    DeleteLogsAfterInput(String),
    DeleteLogsAfterConfirm,
    HotkeyInput(String),
    HotkeyConfirm,
    TickLengthInput(String),
    TickLengthConfirm,
    PortInput(String),
    PortConfirm,
    ServerAddressInput(String),
    ServerAddressConfirm,
}

impl UampApp {
    pub(super) fn settings_event(&mut self, msg: SetMessage) -> ComMsg {
        match msg {
            SetMessage::ExtensionInput(s) => {
                self.gui.set_state.extension_state = s
            }
            SetMessage::ExtensionConfirm => {
                let s = replace(
                    &mut self.gui.set_state.extension_state,
                    String::new(),
                );
                return ComMsg::Msg(Msg::Config(
                    ConfMessage::AddAudioExtension(s),
                ));
            }
            SetMessage::SearchPathInput(s) => {
                self.gui.set_state.search_path_state = s
            }
            SetMessage::SearchPathConfirm => {
                let s = replace(
                    &mut self.gui.set_state.search_path_state,
                    String::new(),
                );
                return ComMsg::Msg(Msg::Config(ConfMessage::AddSearchPath(
                    s.into(),
                )));
            }
            SetMessage::VolumeJumpInput(s) => {
                self.gui.set_state.volume_jump_state = s
            }
            SetMessage::VolumeJumpConfirm => {
                let s = replace(
                    &mut self.gui.set_state.volume_jump_state,
                    String::new(),
                );
                match s.parse::<f32>() {
                    Ok(f) => {
                        return ComMsg::Msg(Msg::Config(
                            ConfMessage::VolumeJump(f / 100.),
                        ))
                    }
                    Err(e) => {
                        error!("Failed to parse volume jump: {e}");
                    }
                }
            }
            SetMessage::SaveTimeoutInput(s) => {
                self.gui.set_state.save_timeout_state = s
            }
            SetMessage::SaveTimeoutConfirm => {
                let s = replace(
                    &mut self.gui.set_state.save_timeout_state,
                    String::new(),
                );
                if s.is_empty() {
                    return ComMsg::Msg(Msg::Config(
                        ConfMessage::SaveTimeout(None),
                    ));
                }
                match str_to_duration(&s) {
                    Some(d) => {
                        return ComMsg::Msg(Msg::Config(
                            ConfMessage::SaveTimeout(Some(d)),
                        ))
                    }
                    None => {
                        error!("Failed to parse save timeout");
                    }
                }
            }
            SetMessage::SeekJumpInput(s) => {
                self.gui.set_state.seek_jump_state = s
            }
            SetMessage::SeekJumpConfirm => {
                let s = replace(
                    &mut self.gui.set_state.seek_jump_state,
                    String::new(),
                );
                match str_to_duration(&s) {
                    Some(d) => {
                        return ComMsg::Msg(Msg::Config(
                            ConfMessage::SeekJump(d),
                        ))
                    }
                    None => {
                        error!("Failed to parse seek jump");
                    }
                }
            }
            SetMessage::DeleteLogsAfterInput(s) => {
                self.gui.set_state.delete_logs_after_state = s
            }
            SetMessage::DeleteLogsAfterConfirm => {
                let s = replace(
                    &mut self.gui.set_state.delete_logs_after_state,
                    String::new(),
                );
                match str_to_duration(&s) {
                    Some(d) => {
                        return ComMsg::Msg(Msg::Config(
                            ConfMessage::DeleteLogsAfter(d),
                        ))
                    }
                    None => {
                        error!("Failed to parse log timeout");
                    }
                }
            }
            SetMessage::HotkeyInput(s) => self.gui.set_state.hotkey_state = s,
            SetMessage::HotkeyConfirm => {
                let s = replace(
                    &mut self.gui.set_state.hotkey_state,
                    String::new(),
                );
                let s = s.split(':').map(|s| s.trim()).collect_vec();
                return ComMsg::Msg(Msg::Config(
                    ConfMessage::AddGlobalHotkey(
                        s[0].to_string(),
                        s[1].to_string(),
                    ),
                ));
            }
            SetMessage::TickLengthInput(s) => {
                self.gui.set_state.tick_length_state = s
            }
            SetMessage::TickLengthConfirm => {
                let s = replace(
                    &mut self.gui.set_state.tick_length_state,
                    String::new(),
                );
                match str_to_duration(&s) {
                    Some(d) => {
                        return ComMsg::Msg(Msg::Config(
                            ConfMessage::TickLength(d),
                        ))
                    }
                    None => {
                        error!("Failed to parse tick length");
                    }
                }
            }
            SetMessage::PortInput(s) => self.gui.set_state.port_state = s,
            SetMessage::PortConfirm => {
                let s =
                    replace(&mut self.gui.set_state.port_state, String::new());
                match s.parse::<u16>() {
                    Ok(u) => {
                        return ComMsg::Msg(Msg::Config(ConfMessage::Port(u)))
                    }
                    Err(e) => {
                        error!("Failed to parse server port: {e}");
                    }
                }
            }
            SetMessage::ServerAddressInput(s) => {
                self.gui.set_state.server_address_state = s
            }
            SetMessage::ServerAddressConfirm => {
                let s = replace(
                    &mut self.gui.set_state.server_address_state,
                    String::new(),
                );
                return ComMsg::Msg(Msg::Config(ConfMessage::ServerAddress(
                    s,
                )));
            }
        }

        ComMsg::none()
    }

    pub(super) fn settings_page(&self) -> Element {
        col![
            container(row![text("Settings")
                .width(300)
                .size(40)
                .vertical_alignment(Vertical::Center)
                .style(Text::Default)
                .font(Font {
                    weight: Weight::Semibold,
                    ..Default::default()
                }),],)
            .padding([5, 20, 5, 20])
            .height(80)
            .style(Container::TopGrad),
            self.items()
        ]
        .into()
    }

    fn items(&self) -> Element {
        wrap_box![
            &self.gui.wb_states[WB_SETTINGS],
            the_button("Search for new songs")
                .on_press(Msg::Control(ControlMsg::LoadNewSongs)),
            toggle(
                "Recursive search for new songs",
                self.config.recursive_search(),
                ConfMessage::RecursiveSearch
            ),
            toggle(
                "Update library on start",
                self.config.update_library_on_start(),
                ConfMessage::UpdateLibraryOnStart,
            ),
            title("Library search paths"),
            delete_list(
                self.config
                    .search_paths()
                    .iter()
                    .map(|p| p.to_string_lossy()),
                ConfMessage::RemoveSearchPath
            ),
            container(add_input(
                "path",
                &self.gui.set_state.search_path_state,
                SetMessage::SearchPathInput,
                |_| true,
                SetMessage::SearchPathConfirm,
                icons::ADD,
                EmptyBehaviour::Ignore,
            ))
            .width(400)
            .height(Shrink)
            .padding([0, 0, 0, 25]),
            title("Song extensions"),
            delete_list(
                self.config.audio_extensions().iter().map(|p| p.into()),
                ConfMessage::RemoveAudioExtension
            ),
            container(add_input(
                "extension",
                &self.gui.set_state.extension_state,
                SetMessage::ExtensionInput,
                |s| !s.find('.').is_some(),
                SetMessage::ExtensionConfirm,
                icons::ADD,
                EmptyBehaviour::Ignore,
            ))
            .width(200)
            .height(Shrink)
            .padding([0, 0, 0, 25]),
            toggle(
                "Global hotkeys",
                self.config.register_global_hotkeys(),
                ConfMessage::RegisterGlobalHotkeys
            ),
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
            toggle(
                "Enable server for CLI",
                self.config.enable_server(),
                ConfMessage::EnableServer,
            ),
            col![
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

fn toggle<'a, M>(s: &'static str, value: bool, msg: M) -> wid::CursorGrad<'a>
where
    M: Fn(bool) -> ConfMessage + 'static,
{
    cursor_grad(
        switch(
            line_text(s)
                .vertical_alignment(Vertical::Center)
                .style(Text::NoForeground)
                .width(Shrink),
            value,
        )
        .padding([0, 0, 0, 10])
        .on_toggle(move |b| Some(Msg::Config(msg(b))))
        .width(Shrink)
        .height(Fill),
    )
    .padding([0, 10, 0, 10])
    .width(Shrink)
    .height(30)
}

fn title<'a>(s: &'static str) -> wid::Container<'a> {
    container(
        line_text(s)
            .vertical_alignment(Vertical::Bottom)
            .width(Shrink)
            .height(Fill),
    )
    .width(Shrink)
    .height(30)
    .padding([0, 10, 0, 10])
}

fn delete_list<'a, I, F>(items: I, del: F) -> wid::Column<'a>
where
    I: Iterator<Item = Cow<'a, str>>,
    F: Fn(usize) -> ConfMessage,
{
    column(
        items
            .enumerate()
            .map(|(i, s)| {
                cursor_grad(
                    row![
                        svg_button(icons::TRASH)
                            .on_click(Msg::Config(del(i)))
                            .padding(6)
                            .style(SvgButton::RedHover)
                            .width(30),
                        line_text(s)
                            .elipsis("...")
                            .vertical_alignment(Vertical::Center)
                            .width(Shrink)
                            .padding([0, 0, 0, 4])
                    ]
                    .width(Shrink),
                )
                .height(30)
                .width(Shrink)
                .padding([0, 10, 0, 4])
                .into()
            })
            .collect(),
    )
    .padding([0, 0, 0, 25])
    .height(Shrink)
    .spacing(5)
}

#[derive(Copy, Clone, Debug)]
enum EmptyBehaviour {
    Allow,
    Ignore,
    _Invalid,
}

fn add_input<'a, F, C, I>(
    placeholder: &'a str,
    text: &'a str,
    change: C,
    validator: F,
    confirm: SetMessage,
    icon: I,
    empty: EmptyBehaviour,
) -> wid::CursorGrad<'a>
where
    F: Fn(&'a str) -> bool,
    C: Fn(String) -> SetMessage + 'a,
    I: Into<svg::Handle>,
{
    let valid = match empty {
        EmptyBehaviour::_Invalid => !text.is_empty() && validator(text),
        _ => text.is_empty() || validator(text),
    };

    let mut but =
        svg_button(icon)
            .width(30)
            .height(Fill)
            .padding(6)
            .style(if valid {
                SvgButton::TransparentCircle(6.)
            } else {
                SvgButton::RedHover
            });
    let mut input = text_input(placeholder, text)
        .style(if valid {
            TextInput::Default
        } else {
            TextInput::Invalid
        })
        .on_input(move |s| Msg::Gui(GuiMessage::Setings(change(s))));

    let act = match empty {
        EmptyBehaviour::Allow => valid || text.is_empty(),
        _ => valid && !text.is_empty(),
    };

    if act {
        but = but.on_click(Msg::Gui(GuiMessage::Setings(confirm.clone())));
        input = input.on_submit(Msg::Gui(GuiMessage::Setings(confirm)))
    }

    cursor_grad(row![but, input,].padding([0, 0, 0, 4]))
        .width(Fill)
        .height(30)
}
