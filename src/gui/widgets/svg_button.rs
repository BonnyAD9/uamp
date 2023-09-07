use std::default;

use iced::widget::Svg;
use iced_core::{
    event::Status,
    layout::{Limits, Node},
    mouse::{self, Cursor},
    renderer::{Quad, Style},
    svg::{self, Handle},
    widget::{tree, Tree},
    Background, BorderRadius, Color, Element, Event, Layout, Length,
    Rectangle, Size, Vector, Widget,
};

use super::sides::Sides;

pub struct SvgButton<Message, Renderer>
where
    Renderer: svg::Renderer,
    Renderer::Theme: StyleSheet,
    Message: Clone,
{
    width: Length,
    height: Length,
    padding: Sides<f32>,
    svg: Handle,
    on_click: Option<Message>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<Message, Renderer> SvgButton<Message, Renderer>
where
    Renderer: svg::Renderer,
    Renderer::Theme: StyleSheet,
    Message: Clone,
{
    /// Creates new border
    pub fn new(svg: Handle) -> Self {
        Self {
            width: Length::Fill,
            height: Length::Fill,
            padding: 0.into(),
            svg,
            on_click: None,
            style: Default::default(),
        }
    }

    /// Sets the width of the border
    pub fn width<L>(mut self, width: L) -> Self
    where
        L: Into<Length>,
    {
        self.width = width.into();
        self
    }

    /// Sets the height of the border
    pub fn height<L>(mut self, height: L) -> Self
    where
        L: Into<Length>,
    {
        self.height = height.into();
        self
    }

    pub fn padding<S>(mut self, padding: S) -> Self
    where
        S: Into<Sides<f32>>,
    {
        self.padding = padding.into();
        self
    }

    /// Sets the height of the border
    pub fn style(
        mut self,
        style: <Renderer::Theme as StyleSheet>::Style,
    ) -> Self {
        self.style = style;
        self
    }

    pub fn on_click(mut self, msg: Message) -> Self {
        self.on_click = Some(msg);
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer>
    for SvgButton<Message, Renderer>
where
    Renderer: svg::Renderer,
    Renderer::Theme: StyleSheet,
    Message: Clone,
{
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, _renderer: &Renderer, limits: &Limits) -> Node {
        let lim = limits.width(self.width).height(self.height);
        Node::new(lim.fill())
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced_core::Clipboard,
        shell: &mut iced_core::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> Status {
        let bounds = layout.bounds();
        let state = state.state.downcast_mut::<State>();

        if !cursor.is_over(bounds) {
            state.pressed = false;
            return Status::Ignored;
        }

        if let Event::Mouse(event) = event {
            match event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    state.pressed = true;
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    state.pressed = false;
                    if let Some(msg) = &self.on_click {
                        shell.publish(msg.clone());
                        return Status::Captured;
                    }
                }
                _ => {}
            }
        }

        Status::Ignored
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced_core::Renderer>::Theme,
        _style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = state.state.downcast_ref::<State>();

        let ap = if state.pressed {
            theme.pressed(&self.style)
        } else if cursor.is_over(bounds) {
            theme.hovered(&self.style)
        } else {
            theme.active(&self.style)
        };

        let svg_bounds = Rectangle {
            x: bounds.x + self.padding.left,
            y: bounds.y + self.padding.top,
            width: bounds.width - self.padding.left - self.padding.right,
            height: bounds.height - self.padding.top - self.padding.bottom,
        };

        renderer.draw(self.svg.clone(), ap.svg_color, svg_bounds);

        let quad = Quad {
            bounds,
            border_radius: ap.border_radius.into(),
            border_width: ap.border_thickness,
            border_color: ap.border_color,
        };

        renderer.fill_quad(quad, ap.background);
    }
}

impl<'a, Message, Renderer> From<SvgButton<Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: svg::Renderer + 'a,
    Renderer::Theme: StyleSheet,
    Message: Clone + 'a,
{
    fn from(value: SvgButton<Message, Renderer>) -> Self {
        Self::new(value)
    }
}

pub struct Appearance {
    pub background: Background,
    pub border_color: Color,
    pub border_radius: Sides<f32>,
    pub border_thickness: f32,
    pub svg_color: Option<Color>,
}

pub trait StyleSheet {
    type Style: Default;

    fn active(&self, style: &Self::Style) -> Appearance;
    fn hovered(&self, style: &Self::Style) -> Appearance;
    fn pressed(&self, style: &Self::Style) -> Appearance;
}

#[derive(Default, Clone)]
struct State {
    pressed: bool,
}
