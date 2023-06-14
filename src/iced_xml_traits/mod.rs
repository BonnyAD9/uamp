use std::{borrow::Cow, ops::RangeInclusive};

use iced::{
    overlay,
    widget::{
        button, checkbox, container, image, mouse_area::MouseArea, pane_grid,
        pick_list, scrollable, text, progress_bar, Button, Checkbox, Column, Container,
        Image, PaneGrid, PickList, ProgressBar, Space,
    },
};
use iced_native::{widget::pane_grid::Pane, Element, Length};

pub trait Empty {
    fn empty() -> Self;
}

pub trait Text {
    fn with_text<STR: ToString>(text: STR) -> Self;
}

pub trait Child<'a, Message, Renderer: iced_native::Renderer> {
    fn with_child(child: impl Into<Element<'a, Message, Renderer>>) -> Self;
}

pub trait Children<'a, Message, Renderer: iced_native::Renderer> {
    fn with_children(children: Vec<Element<'a, Message, Renderer>>) -> Self;
}

pub trait Content<T> {
    fn with_content(content: T) -> Self;
}

// Button

impl<'a, Message, Renderer> Empty for Button<'a, Message, Renderer>
where
    Renderer: 'a + iced_native::Renderer + iced_native::text::Renderer,
    Renderer::Theme: button::StyleSheet + text::StyleSheet,
    Message: 'a
{
    fn empty() -> Self {
        button::<'a, Message, Renderer>(Space::new(Length::Fill, Length::Fill))
    }
}

impl<'a, Message, Renderer> Text for Button<'a, Message, Renderer>
where
    Renderer: 'a + iced_native::Renderer + iced_native::text::Renderer,
    Renderer::Theme: button::StyleSheet + text::StyleSheet,
{
    fn with_text<STR: ToString>(s: STR) -> Self {
        button::<'a, Message, Renderer>(text::<'a, Renderer>(s))
    }
}

impl<'a, Message, Renderer> Child<'a, Message, Renderer>
    for Button<'a, Message, Renderer>
where
    Renderer: 'a + iced_native::Renderer,
    Renderer::Theme: button::StyleSheet,
{
    fn with_child(child: impl Into<Element<'a, Message, Renderer>>) -> Self {
        button::<'a, Message, Renderer>(child)
    }
}

impl<'a, Message, Renderer, E> Content<E> for Button<'a, Message, Renderer>
where
    Renderer: 'a + iced_native::Renderer,
    Renderer::Theme: button::StyleSheet,
    E: Into<Element<'a, Message, Renderer>>,
{
    fn with_content(content: E) -> Self {
        button::<'a, Message, Renderer>(content)
    }
}

// CheckBox

impl<'a, Message, Renderer, STR, F> Content<(STR, bool, F)>
    for Checkbox<'a, Message, Renderer>
where
    Renderer: 'a + iced_native::Renderer + iced_native::text::Renderer,
    Renderer::Theme: checkbox::StyleSheet + text::StyleSheet,
    STR: ToString,
    F: Fn(bool) -> Message + 'a,
{
    fn with_content(content: (STR, bool, F)) -> Self {
        checkbox::<'a, Message, Renderer>(
            content.0.to_string(),
            content.1,
            content.2,
        )
    }
}

// Column

impl<'a, Message, Renderer> Child<'a, Message, Renderer>
    for Column<'a, Message, Renderer>
where
    Renderer: 'a + iced_native::Renderer,
{
    fn with_child(child: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Column::<'a, Message, Renderer>::with_children(vec![child.into()])
    }
}

impl<'a, Message, Renderer> Children<'a, Message, Renderer>
    for Column<'a, Message, Renderer>
where
    Renderer: 'a + iced_native::Renderer,
{
    fn with_children(children: Vec<Element<'a, Message, Renderer>>) -> Self {
        Column::<'a, Message, Renderer>::with_children(children)
    }
}

impl<'a, Message, Renderer> Content<Vec<Element<'a, Message, Renderer>>>
    for Column<'a, Message, Renderer>
where
    Renderer: 'a + iced_native::Renderer,
{
    fn with_content(content: Vec<Element<'a, Message, Renderer>>) -> Self {
        Column::<'a, Message, Renderer>::with_children(content)
    }
}

// Container

impl<'a, Message, Renderer> Child<'a, Message, Renderer>
    for Container<'a, Message, Renderer>
where
    Renderer: 'a + iced_native::Renderer,
    Renderer::Theme: container::StyleSheet,
{
    fn with_child(child: impl Into<Element<'a, Message, Renderer>>) -> Self {
        container::<'a, Message, Renderer>(child)
    }
}

impl<'a, Message, Renderer, T> Content<T> for Container<'a, Message, Renderer>
where
    Renderer: 'a + iced_native::Renderer,
    Renderer::Theme: container::StyleSheet,
    T: Into<Element<'a, Message, Renderer>>,
{
    fn with_content(content: T) -> Self {
        container::<'a, Message, Renderer>(content)
    }
}

// Image

impl<IMG> Content<IMG> for Image
where
    IMG: Into<image::Handle>,
{
    fn with_content(content: IMG) -> Self {
        image(content)
    }
}

// MouseArea

impl<'a, Message, Renderer> Child<'a, Message, Renderer>
    for MouseArea<'a, Message, Renderer>
where
    Renderer: 'a + iced_native::Renderer,
{
    fn with_child(child: impl Into<Element<'a, Message, Renderer>>) -> Self {
        MouseArea::<'a, Message, Renderer>::new(child)
    }
}

impl<'a, Message, Renderer, E> Content<E> for MouseArea<'a, Message, Renderer>
where
    Renderer: 'a + iced_native::Renderer,
    E: Into<Element<'a, Message, Renderer>>,
{
    fn with_content(content: E) -> Self {
        MouseArea::<'a, Message, Renderer>::new(content)
    }
}

// PaneGrid

impl<'a, Message, Renderer, T, F> Content<(&'a pane_grid::State<T>, F)>
    for PaneGrid<'a, Message, Renderer>
where
    Renderer: 'a + iced_native::Renderer,
    Renderer::Theme: pane_grid::StyleSheet + container::StyleSheet,
    T: 'a,
    F: Fn(Pane, &'a T, bool) -> pane_grid::Content<'a, Message, Renderer>,
{
    fn with_content(content: (&'a pane_grid::State<T>, F)) -> Self {
        PaneGrid::<'a, Message, Renderer>::new(content.0, content.1)
    }
}

// PickList

impl<'a, Message, Renderer, T, OPT, F> Content<(OPT, Option<T>, F)>
    for PickList<'a, T, Message, Renderer>
where
    Renderer: 'a + iced_native::text::Renderer,
    Renderer::Theme: pick_list::StyleSheet
        + overlay::menu::StyleSheet
        + scrollable::StyleSheet
        + container::StyleSheet,
    <Renderer::Theme as overlay::menu::StyleSheet>::Style:
        From<<Renderer::Theme as pick_list::StyleSheet>::Style>,
    [T]: ToOwned<Owned = Vec<T>>,
    OPT: Into<Cow<'a, [T]>>,
    F: Fn(T) -> Message + 'a,
    T: ToString + Eq + 'static,
{
    fn with_content(content: (OPT, Option<T>, F)) -> Self {
        pick_list::<'a, Message, Renderer, T>(content.0, content.1, content.2)
    }
}

// ProgressBar

impl<Renderer> Content<(RangeInclusive<f32>, f32)> for ProgressBar<Renderer>
where Renderer: iced_native::Renderer,
    Renderer::Theme: progress_bar::StyleSheet,
{
    fn with_content(content: (RangeInclusive<f32>, f32)) -> Self {
        progress_bar(content.0, content.1)
    }
}
