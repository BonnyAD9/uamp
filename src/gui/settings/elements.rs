use std::borrow::Cow;

use iced_core::{alignment::Vertical, Length::{Shrink, Fill}, svg};

use crate::{gui::{wid::{self, cursor_grad, switch, line_text, container, column, svg_button, text_input}, theme::{Text, SvgButton, TextInput}, widgets::icons, GuiMessage}, config::ConfMessage, core::msg::Msg, row};

use super::SetMessage;

pub fn toggle<'a, M>(s: &'static str, value: bool, msg: M) -> wid::CursorGrad<'a>
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

pub fn title<'a>(s: &'static str) -> wid::Container<'a> {
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

pub fn delete_list<'a, I, F>(items: I, del: F) -> wid::Column<'a>
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
pub enum EmptyBehaviour {
    Allow,
    Ignore,
    _Invalid,
}

pub fn add_input<'a, F, C, I>(
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
