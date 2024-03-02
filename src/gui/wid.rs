use std::{borrow::Cow, cell::Cell, ops::RangeInclusive};

use iced::widget;
use iced_core::Length::{self, Fill};

use crate::core::msg::Msg;

use super::{
    theme::Theme,
    widgets::{self, grid::SpanLen},
};

// collection of less generic types

/// Renderer used in uamp
pub type Renderer = iced::Renderer;
/// Element used in uamp
pub type Element<'a> = iced::Element<'a, Msg, Theme, Renderer>;
/// Command used in uamp
pub type Command = iced::Command<Msg>;
/// Subscription use in uamp
pub type Subscription = iced::Subscription<Msg>;

/// WrapBox widget as used in uamp
pub type WrapBox<'a> = widgets::wrap_box::WrapBox<'a, Msg, Theme, Renderer>;
/// Button widget as used in uamp
pub type Button<'a> = widget::Button<'a, Msg, Theme, Renderer>;
/// Button widget as used in uamp
pub type SvgButton = widgets::svg_button::SvgButton<Msg, Theme>;
/// Text widget as used in uamp
pub type Text<'a> = widget::Text<'a, Theme, Renderer>;
/// Text widget as used in uamp
pub type LineText<'a> = widgets::line_text::LineText<'a, Theme, Renderer>;
/// Column widget as used in uamp
pub type Column<'a> = widget::Column<'a, Msg, Theme, Renderer>;
/// Row widget as used in uamp
pub type Row<'a> = widget::Row<'a, Msg, Theme, Renderer>;
/// Svg widget as used in uamp
pub type Svg = widget::Svg<Theme>;
/// Space widget as used in uamp
pub type Space = widget::Space;
/// Slider widget as used in uamp
pub type Slider<'a, T> = widget::Slider<'a, T, Msg, Theme>;
/// Container widget as used in uamp
pub type Container<'a> = widget::Container<'a, Msg, Theme, Renderer>;
/// Border widget used by uamp
pub type Border<'a> = widgets::border::Border<'a, Msg, Theme, Renderer>;
/// Border widget used by uamp
pub type CursorGrad<'a> = widgets::cursor_grad::CursorGrad<'a, Msg, Theme, Renderer>;
/// Grid
pub type Grid<'a> = widgets::grid::Grid<'a, Msg, Theme, Renderer>;
/// Item in a grid
pub type GridItem<'a> = widgets::grid::GridItem<'a, Msg, Theme, Renderer>;
/// Toggler
pub type Switch<'a> = widgets::switch::Switch<'a, Msg, Theme, Renderer>;
pub type TextInput<'a> = widget::TextInput<'a, Msg, Theme, Renderer>;
pub type MouseInteraction<'a> =
    widgets::mouse_interaction::MouseInteraction<'a, Msg, Theme, Renderer>;
pub type Image = widget::Image<iced_core::image::Handle>;

pub type WrapBoxState = Cell<widgets::wrap_box::State>;

/// creates wrap_box widhet with the given children
pub fn wrap_box<'a>(
    children: Vec<Element<'a>>,
    state: &'a WrapBoxState,
) -> WrapBox<'a> {
    WrapBox::with_children(children, state)
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
    ($s:expr, $($x:expr),+ $(,)?) => (
        $crate::gui::wid::wrap_box(vec![$($crate::gui::wid::Element::from($x)),+], $s)
    );
}

/// Creates button widget with the given child
pub fn button<'a>(child: impl Into<Element<'a>>) -> Button<'a> {
    Button::new(child).width(Fill).height(Fill).padding(0)
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
        crate::gui::wid::button(text!($s))
    };
    ($fmt:literal, $($args:expr),+) => {
        crate::wid::button(text!($fmt, $($args),+)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill))
    };
}

/// Creates text widget with the given string content
pub fn text<'a>(content: impl Into<Cow<'a, str>>) -> Text<'a> {
    Text::new(content).width(Fill).height(Fill)
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
        crate::gui::wid::text("")
    };
    ($s:expr) => {
        $crate::gui::wid::text($s)
    };
    ($fmt:literal, $($args:expr),+) => {
        $crate::gui::wid::text(format!($fmt, $($args),+))
    };
}

/// Creates column widget with the given children
pub fn column<'a>(children: Vec<Element<'a>>) -> Column<'a> {
    Column::with_children(children).width(Fill).height(Fill)
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
        $crate::gui::wid::Column::new($crate::gui::wid::nothing())
    );
    ($($x:expr),+ $(,)?) => (
        $crate::gui::wid::column(vec![$($crate::gui::wid::Element::from($x)),+])
    );
}

/// Creates row widget with the given children
pub fn row<'a>(children: Vec<Element<'a>>) -> Row<'a> {
    Row::with_children(children).width(Fill).height(Fill)
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
        $crate::gui::wid::Row::new($crate::gui::wid::nothing())
    );
    ($($x:expr),+ $(,)?) => {{
        $crate::gui::wid::row(vec![$($crate::gui::wid::Element::from($x)),+])
    }};
}

/// Creates svg widget
pub fn svg(handle: impl Into<widget::svg::Handle>) -> Svg {
    Svg::new(handle).height(Fill)
}

/// Creates space widget
pub fn space(width: impl Into<Length>, height: impl Into<Length>) -> Space {
    Space::new(width, height)
}

/// Creates widget that fills
pub fn nothing() -> Space {
    space(Fill, Fill)
}

/// Creates slider widget
pub fn slider<'a, T: Copy + From<u8> + std::cmp::PartialOrd>(
    range: RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> Msg + 'a,
) -> Slider<'a, T> {
    widget::slider(range, value, on_change)
}

/// Creates container widget with the given child
pub fn container<'a>(child: impl Into<Element<'a>>) -> Container<'a> {
    widget::container(child).width(Fill).height(Fill)
}

pub fn border<'a, E>(child: E) -> Border<'a>
where
    E: Into<Element<'a>>,
{
    Border::new(child.into())
}

pub fn svg_button(handle: impl Into<widget::svg::Handle>) -> SvgButton {
    SvgButton::new(handle.into())
}

#[macro_export]
macro_rules! grid {
    ($($col:expr),* $(,)? ; $($row:expr),* ; $($elem:expr),+ $(,)?) => {
        $crate::gui::wid::grid(
            [$($col),*].iter().map(|i| *i),
            [$($row),*].iter().map(|i| *i),
            vec![$($crate::gui::wid::GridItem::from($elem)),*]
        )
    };
}

pub fn grid<'a, I1, I2>(
    columns: I1,
    rows: I2,
    items: Vec<GridItem<'a>>,
) -> Grid<'a>
where
    I1: Iterator<Item = SpanLen>,
    I2: Iterator<Item = SpanLen>,
{
    Grid::new(columns, rows, items)
}

/// Creates text widget with the given string content
pub fn line_text<'a>(content: impl Into<Cow<'a, str>>) -> LineText<'a> {
    LineText::new(content).width(Fill).height(Fill)
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
macro_rules! line_text {
    () => {
        crate::gui::wid::line_text("")
    };
    ($s:expr) => {
        $crate::gui::wid::line_text($s)
    };
    ($fmt:literal, $($args:expr),+) => {
        $crate::gui::wid::line_text(format!($fmt, $($args),+))
    };
}

pub fn cursor_grad<'a, E>(child: E) -> CursorGrad<'a>
where
    E: Into<Element<'a>>,
{
    CursorGrad::new(child.into())
}

pub fn switch<'a, E>(child: E, is_toggled: bool) -> Switch<'a>
where
    E: Into<Element<'a>>,
{
    Switch::new(child, is_toggled)
}

pub fn text_input<'a>(placeholder: &'a str, value: &'a str) -> TextInput<'a> {
    TextInput::new(placeholder, value)
}

pub fn mouse_int<'a, E>(child: E) -> MouseInteraction<'a>
where
    E: Into<Element<'a>>,
{
    MouseInteraction::new(child.into())
}

pub fn image(handle: impl Into<iced_core::image::Handle>) -> Image {
    Image::new(handle)
}
