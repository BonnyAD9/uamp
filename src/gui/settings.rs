use std::{borrow::Cow, mem::replace};

use iced_core::{
    alignment::Vertical,
    font::Weight,
    Font,
    Length::{Fill, Shrink},
};

use crate::{
    app::UampApp,
    col,
    config::ConfMessage,
    core::msg::{ComMsg, ControlMsg, Msg},
    gui::ids::WB_SETTINGS,
    row, wrap_box,
};

use super::{
    elements::the_button,
    theme::{Container, SvgButton, Text, TextInput},
    wid::{
        self, column, container, cursor_grad, line_text, svg_button, switch,
        text, text_input, Element,
    },
    widgets::icons,
    GuiMessage,
};

#[derive(Default)]
pub struct SetState {
    extension_state: String,
    search_path_state: String,
}

#[derive(Clone, Debug)]
pub enum SetMessage {
    ExtensionInput(String),
    ExtensionConfirm,
    SearchPathInput(String),
    SearchPathConfirm,
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
                SetMessage::SearchPathConfirm
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
                SetMessage::ExtensionConfirm
            ))
            .width(200)
            .height(Shrink)
            .padding([0, 0, 0, 25]),
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
            .font(Font {
                weight: Weight::Semibold,
                ..Default::default()
            })
            .vertical_alignment(Vertical::Bottom)
            .size(20)
            .width(Shrink)
            .height(Fill),
    )
    .width(Shrink)
    .height(40)
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

fn add_input<'a, F, C>(
    placeholder: &'static str,
    text: &'a str,
    change: C,
    validator: F,
    confirm: SetMessage,
) -> wid::CursorGrad<'a>
where
    F: Fn(&'a str) -> bool,
    C: Fn(String) -> SetMessage + 'a,
{
    let valid = text.is_empty() || validator(text);

    let mut but = svg_button(icons::ADD)
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

    if valid && !text.is_empty() {
        but = but.on_click(Msg::Gui(GuiMessage::Setings(confirm.clone())));
        input = input.on_submit(Msg::Gui(GuiMessage::Setings(confirm)))
    }

    cursor_grad(row![but, input,].padding([0, 0, 0, 4]))
        .width(Fill)
        .height(30)
}
