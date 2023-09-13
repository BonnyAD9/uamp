use iced::Background;
use iced_core::{
    alignment::Vertical,
    event::Status,
    layout::{Limits, Node},
    mouse::{self, Button},
    renderer::Quad,
    text,
    widget::Tree,
    Color, Element, Event, Layout, Length, Rectangle, Size, Vector, Widget,
};

use super::{limit_size, sides::Sides};

const SWITCH_SIZE: f32 = 20.;

pub struct Switch<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    child: Element<'a, Message, Renderer>,
    width: Length,
    height: Length,
    padding: Sides<f32>,
    is_active: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> Option<Message>>>,
    alignment: Vertical,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> Switch<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    pub fn new<E>(child: E, is_active: bool) -> Self
    where
        E: Into<Element<'a, Message, Renderer>>,
    {
        Self {
            child: child.into(),
            width: Length::Fill,
            height: Length::Shrink,
            padding: 0.into(),
            is_active,
            on_toggle: None,
            alignment: Vertical::Center,
            style: Default::default(),
        }
    }

    pub fn width<L>(mut self, width: L) -> Self
    where
        L: Into<Length>,
    {
        self.width = width.into();
        self
    }

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

    pub fn on_toggle<F>(mut self, f: F) -> Self
    where
        F: Fn(bool) -> Option<Message> + 'static,
    {
        self.on_toggle = Some(Box::new(f));
        self
    }

    pub fn alignment(mut self, align: Vertical) -> Self {
        self.alignment = align;
        self
    }

    pub fn style(
        mut self,
        style: <Renderer::Theme as StyleSheet>::Style,
    ) -> Self {
        self.style = style;
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Switch<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.child)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.child])
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &Limits) -> Node {
        let lim = limits
            .min_width(SWITCH_SIZE * 2.)
            .min_height(SWITCH_SIZE)
            .width(self.width)
            .height(self.height);

        let child_limits = lim.pad(
            Sides {
                left: self.padding.left + SWITCH_SIZE * 2.,
                ..self.padding
            }
            .into(),
        );

        let child = self
            .child
            .as_widget()
            .layout(renderer, &child_limits)
            .translate(Vector::new(
                self.padding.left + SWITCH_SIZE * 2.,
                self.padding.top,
            ));

        let child_size = child.size();
        let min_size = lim.min();

        let lim = lim
            .min_width(
                min_size.width + child_size.width + self.padding.lr_sum(),
            )
            .min_height(
                min_size.height + child_size.height + self.padding.tb_sum(),
            );

        Node::with_children(
            limit_size(&lim, self.width, self.height),
            vec![child],
        )
    }

    fn operate(
        &self,
        state: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced_core::widget::Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.child.as_widget().operate(
                &mut state.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            )
        })
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: iced_core::Event,
        layout: Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced_core::Clipboard,
        shell: &mut iced_core::Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> Status {
        let mut status = self.child.as_widget_mut().on_event(
            &mut state.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let bounds = layout.bounds();
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonReleased(Button::Left))
        ) && cursor.is_over(bounds)
        {
            if let Some(f) = &self.on_toggle {
                if let Some(msg) = f(!self.is_active) {
                    shell.publish(msg);
                    status = status.merge(Status::Captured);
                }
            }
        }

        status
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> iced_core::mouse::Interaction {
        self.child.as_widget().mouse_interaction(
            &state.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        state: &iced_core::widget::Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced_core::Renderer>::Theme,
        style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced_core::Rectangle,
    ) {
        let mut bounds = layout.bounds();

        let ap = if cursor.is_over(bounds) {
            if self.is_active {
                theme.active_hovered(&self.style)
            } else {
                theme.inactive_hovered(&self.style)
            }
        } else {
            if self.is_active {
                theme.active(&self.style)
            } else {
                theme.inactive(&self.style)
            }
        };

        bounds.y = match self.alignment {
            Vertical::Top => bounds.y,
            Vertical::Center => bounds.y + (bounds.height - SWITCH_SIZE) / 2.,
            Vertical::Bottom => bounds.y + bounds.height - SWITCH_SIZE,
        };

        let rdifx =
            (SWITCH_SIZE - ap.rail_size.width).max(0.).min(SWITCH_SIZE) / 2.;
        let rdify =
            (SWITCH_SIZE - ap.rail_size.height).max(0.).min(SWITCH_SIZE) / 2.;
        let tdifx = (SWITCH_SIZE - ap.thumb_size.width)
            .max(0.)
            .min(SWITCH_SIZE * 2.)
            / 2.;
        let tdify = (SWITCH_SIZE - ap.thumb_size.height)
            .max(0.)
            .min(SWITCH_SIZE)
            / 2.;

        let rbounds = Rectangle {
            x: bounds.x + rdifx,
            y: bounds.y + rdify,
            width: SWITCH_SIZE * 2. - rdifx * 2.,
            height: SWITCH_SIZE - rdify * 2.,
        };

        let tbounds = Rectangle {
            x: bounds.x
                + tdifx
                + if self.is_active { SWITCH_SIZE } else { 0. },
            y: bounds.y + tdify,
            width: SWITCH_SIZE - tdifx * 2.,
            height: SWITCH_SIZE - tdify * 2.,
        };

        let rquad = Quad {
            bounds: rbounds,
            border_radius: ap.rail_border_radius.into(),
            border_width: ap.rail_border_thickness,
            border_color: ap.rail_border_color,
        };

        let tquad = Quad {
            bounds: tbounds,
            border_radius: ap.thumb_border_radius.into(),
            border_width: ap.thumb_border_thickness,
            border_color: ap.thumb_border_color,
        };

        renderer.fill_quad(rquad, ap.rail_color);
        renderer.fill_quad(tquad, ap.thumb_color);

        let style = if let Some(c) = ap.text_color {
            let mut style = style.clone();
            style.text_color = c;
            style
        } else {
            style.clone()
        };

        self.child.as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            &style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<iced_core::overlay::Element<'b, Message, Renderer>> {
        self.child.as_widget_mut().overlay(
            &mut state.children[0],
            layout.children().next().unwrap(),
            renderer,
        )
    }
}

impl<'a, Message, Renderer> From<Switch<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: text::Renderer + 'a,
    Renderer::Theme: StyleSheet,
    Message: 'a,
{
    fn from(value: Switch<'a, Message, Renderer>) -> Self {
        Self::new(value)
    }
}

pub struct Appearance {
    pub rail_size: Size,
    pub thumb_size: Size,
    pub rail_border_color: Color,
    pub thumb_border_color: Color,
    pub rail_border_radius: Sides<f32>,
    pub thumb_border_radius: Sides<f32>,
    pub rail_border_thickness: f32,
    pub thumb_border_thickness: f32,
    pub rail_color: Background,
    pub thumb_color: Background,
    pub text_color: Option<Color>,
}

pub trait StyleSheet {
    type Style: Default;

    fn active(&self, style: &Self::Style) -> Appearance;

    fn inactive(&self, style: &Self::Style) -> Appearance;

    fn active_hovered(&self, style: &Self::Style) -> Appearance;

    fn inactive_hovered(&self, style: &Self::Style) -> Appearance;
}
