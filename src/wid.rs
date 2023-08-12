use std::{borrow::Cow, ops::RangeInclusive};

use iced::widget;
use iced_core::Length::{self, FillPortion, Shrink};

use crate::{fancy_widgets, theme::Theme, uamp_app::UampMessage};

// collection of less generic types

pub type Renderer = iced::Renderer<Theme>;
pub type Element<'a> = iced::Element<'a, UampMessage, Renderer>;
pub type Command = iced::Command<UampMessage>;
pub type WrapBox<'a> =
    fancy_widgets::wrap_box::WrapBox<'a, UampMessage, Renderer>;
pub type Button<'a> = widget::Button<'a, UampMessage, Renderer>;
pub type Text<'a> = widget::Text<'a, Renderer>;
pub type Column<'a> = widget::Column<'a, UampMessage, Renderer>;
pub type Row<'a> = widget::Row<'a, UampMessage, Renderer>;
pub type Svg = widget::Svg<Renderer>;
pub type Space = widget::Space;
pub type Scrollable<'a> = widget::Scrollable<'a, UampMessage, Renderer>;
pub type Slider<'a, T> = widget::Slider<'a, T, UampMessage, Renderer>;
pub type Container<'a> = widget::Container<'a, UampMessage, Renderer>;

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
        crate::wid::button(text!($fmt, $($args),+)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill))
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

pub fn row<'a>(children: Vec<Element<'a>>) -> Row<'a> {
    Row::with_children(children)
}

#[macro_export]
macro_rules! row {
    () => (
        $crate::wid::Row::new(crate::wid::nothing())
    );
    ($($x:expr),+ $(,)?) => {{
        $crate::wid::row(vec![$($crate::wid::Element::from($x)),+])
    }};
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

pub fn slider<'a, T: Copy + From<u8> + std::cmp::PartialOrd>(
    range: RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> UampMessage + 'a,
) -> Slider<'a, T> {
    widget::slider(range, value, on_change)
}

pub fn container<'a>(child: impl Into<Element<'a>>) -> Container<'a> {
    widget::container(child)
}

pub fn center<'a>(child: impl Into<Element<'a>>) -> Row<'a> {
    center_x(center_y(child))
}

pub fn center_x<'a>(child: impl Into<Element<'a>>) -> Row<'a> {
    row![
        space(FillPortion(1), Shrink),
        child.into(),
        space(FillPortion(1), Shrink)
    ]
}

pub fn center_y<'a>(child: impl Into<Element<'a>>) -> Column<'a> {
    col![
        space(Shrink, FillPortion(1)),
        child.into(),
        space(Shrink, FillPortion(1)),
    ]
}
