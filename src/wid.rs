use std::{borrow::Cow, ops::RangeInclusive};

use iced::widget;
use iced_core::Length::{self, FillPortion, Shrink};

use crate::{fancy_widgets, theme::Theme, uamp_app::UampMessage};

// collection of less generic types

/// Renderer used in uamp
pub type Renderer = iced::Renderer<Theme>;
/// Element used in uamp
pub type Element<'a> = iced::Element<'a, UampMessage, Renderer>;
/// Command used in uamp
pub type Command = iced::Command<UampMessage>;
/// WrapBox widget as used in uamp
pub type WrapBox<'a> =
    fancy_widgets::wrap_box::WrapBox<'a, UampMessage, Renderer>;
/// Button widget as used in uamp
pub type Button<'a> = widget::Button<'a, UampMessage, Renderer>;
/// Text widget as used in uamp
pub type Text<'a> = widget::Text<'a, Renderer>;
/// Column widget as used in uamp
pub type Column<'a> = widget::Column<'a, UampMessage, Renderer>;
/// Row widget as used in uamp
pub type Row<'a> = widget::Row<'a, UampMessage, Renderer>;
/// Svg widget as used in uamp
pub type Svg = widget::Svg<Renderer>;
/// Space widget as used in uamp
pub type Space = widget::Space;
/// Scrollable widget as used in uamp
pub type Scrollable<'a> = widget::Scrollable<'a, UampMessage, Renderer>;
/// Slider widget as used in uamp
pub type Slider<'a, T> = widget::Slider<'a, T, UampMessage, Renderer>;
/// Container widget as used in uamp
pub type Container<'a> = widget::Container<'a, UampMessage, Renderer>;

/// creates wrap_box widhet with the given children
pub fn wrap_box<'a>(children: Vec<Element>) -> WrapBox {
    WrapBox::with_childern(children)
}

/// creates wrap_box widhet with the list of children
///
/// # Example
/// ```
/// let wb = wrap_box![
///     widget1,
///     widget2,
/// ];
/// ```
#[macro_export]
macro_rules! wrap_box {
    () => (
        $crate::wid::WrapBox::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::wid::wrap_box(vec![$($Element::from($x)),+])
    );
}

/// Creates button widget with the given child
pub fn button<'a>(child: impl Into<Element<'a>>) -> Button<'a> {
    Button::new(child)
}

/// Creates button widget
///
/// # Examples
/// ```
/// // Empty button
/// button!();
///
/// // button with formatted text
/// button!("Hello {}", name);
/// ```
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

/// Creates text widget with the given string content
pub fn text<'a>(content: impl Into<Cow<'a, str>>) -> Text<'a> {
    Text::new(content)
}

/// Creates text widgets
///
/// # Examples
/// ```
/// // Empty text
/// text!();
///
/// // formatted text
/// text!("hello {}", name);
/// ```
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

/// Creates column widget with the given children
pub fn column<'a>(children: Vec<Element<'a>>) -> Column<'a> {
    Column::with_children(children)
}

/// Creates column widget
///
/// # Examples
/// ```
/// col![
///     wid1,
///     wid2,
/// ];
/// ```
#[macro_export]
macro_rules! col {
    () => (
        $crate::wid::Column::new(crate::wid::nothing())
    );
    ($($x:expr),+ $(,)?) => (
        $crate::wid::column(vec![$($crate::wid::Element::from($x)),+])
    );
}

/// Creates row widget with the given children
pub fn row<'a>(children: Vec<Element<'a>>) -> Row<'a> {
    Row::with_children(children)
}

/// Creates row widget
///
/// # Examples
/// ```
/// row![
///     wid1,
///     wid2,
/// ];
/// ```
#[macro_export]
macro_rules! row {
    () => (
        $crate::wid::Row::new(crate::wid::nothing())
    );
    ($($x:expr),+ $(,)?) => {{
        $crate::wid::row(vec![$($crate::wid::Element::from($x)),+])
    }};
}

/// Creates svg widget
pub fn svg(handle: impl Into<widget::svg::Handle>) -> Svg {
    Svg::new(handle)
}

/// Creates space widget
pub fn space(width: impl Into<Length>, height: impl Into<Length>) -> Space {
    Space::new(width, height)
}

/// Creates widget that shrinks
pub fn nothing() -> Space {
    space(Shrink, Shrink)
}

/// Creates slider widget
pub fn slider<'a, T: Copy + From<u8> + std::cmp::PartialOrd>(
    range: RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> UampMessage + 'a,
) -> Slider<'a, T> {
    widget::slider(range, value, on_change)
}

/// Creates container widget with the given child
pub fn container<'a>(child: impl Into<Element<'a>>) -> Container<'a> {
    widget::container(child)
}

/// Creates container that centers its child
pub fn center<'a>(child: impl Into<Element<'a>>) -> Row<'a> {
    center_x(center_y(child))
}

/// Creates container that centers its child on the x axis
pub fn center_x<'a>(child: impl Into<Element<'a>>) -> Row<'a> {
    row![
        space(FillPortion(1), Shrink),
        child.into(),
        space(FillPortion(1), Shrink)
    ]
}

/// Creates container that centers its child on the y axis
pub fn center_y<'a>(child: impl Into<Element<'a>>) -> Column<'a> {
    col![
        space(Shrink, FillPortion(1)),
        child.into(),
        space(Shrink, FillPortion(1)),
    ]
}
