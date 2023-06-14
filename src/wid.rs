use std::borrow::Cow;

use iced::widget;

use crate::{theme::Theme, uamp_app::UampMessage, fancy_widgets};

// collection of less generic types

pub type Renderer = iced::Renderer<Theme>;
pub type Element<'a> = iced::Element<'a, UampMessage, Renderer>;
pub type Command = iced::Command<UampMessage>;
pub type WrapBox<'a> = fancy_widgets::wrap_box::WrapBox<'a, UampMessage, Renderer>;
pub type Button<'a> = widget::Button<'a, UampMessage, Renderer>;
pub type Text<'a> = widget::Text<'a, Renderer>;
pub type Column<'a> = widget::Column<'a, UampMessage, Renderer>;
pub type Svg = widget::Svg<Renderer>;

pub fn wrap_box<'a>(children: Vec<Element>) -> WrapBox {
    WrapBox::with_childern(children)
}

#[macro_export]
macro_rules! wrap_box {
    () => (
        $WrapBox::new()
    );
    ($($x:expr),+ $(,)?) => (
        $WrapBox::with_children(vec![$($Element::from($x)),+])
    );
}

pub fn button<'a>(child: impl Into<Element<'a>>) -> Button<'a> {
    Button::new(child)
}

pub fn text<'a>(content: impl Into<Cow<'a, str>>) -> Text<'a> {
    Text::new(content)
}

#[macro_export]
macro_rules! text {
    () => {
        crate::wid::text("")
    };
    ($($fmt:literal),+ $(,)?) => (
        $crate::wid::text(format!(fmt,+))
    );
}

pub fn column<'a>(children: Vec<Element<'a>>) -> Column<'a> {
    Column::with_children(children)
}

#[macro_export]
macro_rules! col {
    () => (
        $crate::wid::Column::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::wid::Column::with_children(vec![$($crate::wid::Element::from($x)),+])
    );
}

pub fn svg(handle: impl Into<widget::svg::Handle>) -> Svg {
    Svg::new(handle)
}
