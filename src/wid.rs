use std::{borrow::Cow, fmt::Debug};

use iced::widget;
use iced_native::{
    event::Status,
    Clipboard, Event,
    Length::{self, Shrink},
    Point,
};

use crate::{
    fancy_widgets, library::Library, theme::Theme, uamp_app::UampMessage,
};

// collection of less generic types

pub type Renderer = iced::Renderer<Theme>;
pub type Element<'a> = iced::Element<'a, UampMessage, Renderer>;
pub type Command = iced::Command<UampMessage>;
pub type WrapBox<'a> =
    fancy_widgets::wrap_box::WrapBox<'a, UampMessage, Renderer>;
pub type Button<'a> = widget::Button<'a, UampMessage, Renderer>;
pub type Text<'a> = widget::Text<'a, Renderer>;
pub type Column<'a> = widget::Column<'a, UampMessage, Renderer>;
pub type Svg = widget::Svg<Renderer>;
pub type Space = widget::Space;
pub type EventCapture<'a> =
    fancy_widgets::event_capture::EventCapture<'a, UampMessage>;

pub fn wrap_box<'a>(children: Vec<Element>) -> WrapBox {
    WrapBox::with_childern(children)
}

#[macro_export]
macro_rules! wrap_box {
    () => (
        $crate::wid::WrapBox::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::wid::wrap_box(vec![$($Element::from($x)),+])
    );
}

pub fn button<'a>(child: impl Into<Element<'a>>) -> Button<'a> {
    Button::new(child)
}

#[macro_export]
macro_rules! button {
    () => {
        crate::wid::button(nothing())
    };
    ($s:expr) => {
        crate::wid::button(text!($s))
    };
    ($fmt:literal, $($args:expr),+) => {
        crate::wid::button(text!($fmt, $($args),+))
    };
}

pub fn text<'a>(content: impl Into<Cow<'a, str>>) -> Text<'a> {
    Text::new(content)
}

#[macro_export]
macro_rules! text {
    () => {
        crate::wid::text("")
    };
    ($s:expr) => {
        $crate::wid::text($s)
    };
    ($fmt:literal, $($args:expr),+) => {
        $crate::wid::text(format!($fmt, $($args),+))
    };
}

pub fn column<'a>(children: Vec<Element<'a>>) -> Column<'a> {
    Column::with_children(children)
}

#[macro_export]
macro_rules! col {
    () => (
        $crate::wid::Column::new(crate::wid::nothing())
    );
    ($($x:expr),+ $(,)?) => (
        $crate::wid::column(vec![$($crate::wid::Element::from($x)),+])
    );
}

pub fn svg(handle: impl Into<widget::svg::Handle>) -> Svg {
    Svg::new(handle)
}

pub fn space(width: impl Into<Length>, height: impl Into<Length>) -> Space {
    Space::new(width, height)
}

pub fn nothing() -> Space {
    space(Shrink, Shrink)
}

pub fn event_capture<'a>(
    handle: impl Fn(Event, Point, &mut dyn Clipboard) -> (Option<UampMessage>, Status)
        + 'a,
) -> EventCapture<'a> {
    EventCapture::new(handle)
}
