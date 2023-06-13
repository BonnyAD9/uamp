use iced::widget::{
    button, checkbox, container, image, mouse_area::MouseArea, pane_grid,
    text, Button, Checkbox, Column, Container, Image, PaneGrid,
};
use iced_native::{widget::pane_grid::Pane, Element};

pub trait Text {
    fn with_text<STR: AsRef<str>>(text: STR) -> Self;
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

impl<'a, Message, Renderer> Text for Button<'a, Message, Renderer>
where
    <Renderer as iced_native::Renderer>::Theme: button::StyleSheet,
    <Renderer as iced_native::Renderer>::Theme: text::StyleSheet,
    Renderer: iced_native::text::Renderer,
    Renderer: iced_native::Renderer + 'a,
{
    fn with_text<STR: AsRef<str>>(s: STR) -> Self {
        button::<'a, Message, Renderer>(text::<'a, Renderer>(s.as_ref()))
    }
}

impl<'a, Message, Renderer> Child<'a, Message, Renderer>
    for Button<'a, Message, Renderer>
where
    <Renderer as iced_native::Renderer>::Theme: button::StyleSheet,
    Renderer: iced_native::Renderer + 'a,
{
    fn with_child(child: impl Into<Element<'a, Message, Renderer>>) -> Self {
        button::<'a, Message, Renderer>(child)
    }
}

impl<'a, Message, Renderer, E> Content<E> for Button<'a, Message, Renderer>
where
    <Renderer as iced_native::Renderer>::Theme: button::StyleSheet,
    Renderer: iced_native::Renderer + 'a,
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
    <Renderer as iced_native::Renderer>::Theme: checkbox::StyleSheet,
    <Renderer as iced_native::Renderer>::Theme: text::StyleSheet,
    Renderer: iced_native::Renderer + 'a,
    Renderer: iced_native::text::Renderer,
    STR: Into<String>,
    F: Fn(bool) -> Message + 'a,
{
    fn with_content(content: (STR, bool, F)) -> Self {
        checkbox::<'a, Message, Renderer>(content.0, content.1, content.2)
    }
}

// Column

impl<'a, Message, Renderer> Child<'a, Message, Renderer>
    for Column<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer + 'a,
{
    fn with_child(child: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Column::<'a, Message, Renderer>::with_children(vec![child.into()])
    }
}

impl<'a, Message, Renderer> Children<'a, Message, Renderer>
    for Column<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer + 'a,
{
    fn with_children(children: Vec<Element<'a, Message, Renderer>>) -> Self {
        Column::<'a, Message, Renderer>::with_children(children)
    }
}

impl<'a, Message, Renderer> Content<Vec<Element<'a, Message, Renderer>>>
    for Column<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer + 'a,
{
    fn with_content(content: Vec<Element<'a, Message, Renderer>>) -> Self {
        Column::<'a, Message, Renderer>::with_children(content)
    }
}

// Container

impl<'a, Message, Renderer> Child<'a, Message, Renderer>
    for Container<'a, Message, Renderer>
where
    <Renderer as iced_native::Renderer>::Theme: container::StyleSheet,
    Renderer: iced_native::Renderer + 'a,
{
    fn with_child(child: impl Into<Element<'a, Message, Renderer>>) -> Self {
        container::<'a, Message, Renderer>(child)
    }
}

impl<'a, Message, Renderer, T> Content<T> for Container<'a, Message, Renderer>
where
    <Renderer as iced_native::Renderer>::Theme: container::StyleSheet,
    Renderer: iced_native::Renderer + 'a,
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
    Renderer: iced_native::Renderer,
{
    fn with_child(child: impl Into<Element<'a, Message, Renderer>>) -> Self {
        MouseArea::<'a, Message, Renderer>::new(child)
    }
}

impl<'a, Message, Renderer, E> Content<E> for MouseArea<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
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
    Renderer: iced_native::Renderer,
    <Renderer as iced_native::Renderer>::Theme: pane_grid::StyleSheet,
    <Renderer as iced_native::Renderer>::Theme: container::StyleSheet,
    T: 'a,
    F: Fn(Pane, &'a T, bool) -> pane_grid::Content<'a, Message, Renderer>,
{
    fn with_content(content: (&'a pane_grid::State<T>, F)) -> Self {
        PaneGrid::<'a, Message, Renderer>::new(content.0, content.1)
    }
}
