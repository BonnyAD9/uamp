use iced_core::{
    gradient::Linear,
    layout::{Limits, Node},
    mouse::Cursor,
    renderer::{Quad, Style},
    widget::Tree,
    Background, BorderRadius, Color, Degrees, Element, Gradient, Layout,
    Length, Radians, Rectangle, Size, Vector, Widget,
};

use super::sides::Sides;

pub struct CursorGrad<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
    Renderer::Theme: StyleSheet,
{
    width: Length,
    height: Length,
    padding: Sides<f32>,
    child: Element<'a, Message, Renderer>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> CursorGrad<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates new border
    pub fn new(child: Element<'a, Message, Renderer>) -> Self {
        Self {
            width: Length::Fill,
            height: Length::Fill,
            padding: 0.into(),
            child,
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
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for CursorGrad<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
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
        let lim = limits.width(self.width).height(self.height);
        let size = lim.fill();

        let child_limits = Limits::new(
            Size::new(0., 0.),
            Size::new(
                size.width - self.padding.left - self.padding.right,
                size.height - self.padding.top - self.padding.bottom,
            ),
        );

        Node::with_children(
            size,
            vec![self
                .child
                .as_widget()
                .layout(renderer, &child_limits)
                .translate(Vector::new(self.padding.left, self.padding.top))],
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
    ) -> iced_core::event::Status {
        self.child.as_widget_mut().on_event(
            &mut state.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
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
        state: &Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced_core::Renderer>::Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        let ap = if cursor.is_over(bounds) {
            theme.hovered(&self.style)
        } else {
            theme.active(&self.style)
        };

        let ap = if let Some(ap) = ap {
            ap
        } else {
            self.child.as_widget().draw(
                &state.children[0],
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                viewport,
            );
            return;
        };

        let pos = if let Some(p) = cursor.position() {
            p
        } else {
            self.child.as_widget().draw(
                &state.children[0],
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                viewport,
            );
            return;
        };

        // in pixels
        let mut center = pos.x - bounds.x;
        let left = center - ap.fade_len;
        let right = center + ap.fade_len;

        let (mut left, l_mul) = if left > 0. {
            (left, 0.)
        } else {
            (0., left.abs() / ap.fade_len)
        };

        let (mut right, r_mul) = if right < bounds.width {
            (right, 0.)
        } else {
            (bounds.width, (right - bounds.width) / ap.fade_len)
        };

        center /= bounds.width;
        left /= bounds.width;
        right /= bounds.width;

        let mut grad = Linear::new(Degrees(180.));

        let m = ap.mouse_color;
        let f = ap.fade_color;

        grad = grad.add_stop(
            left,
            Color::from_rgba(
                m.r * l_mul + f.r * (1. - l_mul),
                m.g * l_mul + f.g * (1. - l_mul),
                m.b * l_mul + f.b * (1. - l_mul),
                m.a * l_mul + f.a * (1. - l_mul),
            ),
        );

        grad = grad.add_stop(center, m);

        grad = grad.add_stop(
            right,
            Color::from_rgba(
                m.r * r_mul + f.r * (1. - r_mul),
                m.g * r_mul + f.g * (1. - r_mul),
                m.b * r_mul + f.b * (1. - r_mul),
                m.a * r_mul + f.a * (1. - r_mul),
            ),
        );

        let quad = Quad {
            bounds,
            border_radius: ap.border_radius.into(),
            border_width: 0.,
            border_color: Color::TRANSPARENT,
        };

        renderer.fill_quad(quad, Background::Gradient(Gradient::Linear(grad)));

        self.child.as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            style,
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

impl<'a, Message, Renderer> From<CursorGrad<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
    Renderer::Theme: StyleSheet,
    Message: 'a,
{
    fn from(value: CursorGrad<'a, Message, Renderer>) -> Self {
        Self::new(value)
    }
}

pub struct Appearance {
    pub border_radius: Sides<f32>,
    pub mouse_color: Color,
    pub fade_color: Color,
    pub fade_len: f32,
}

pub trait StyleSheet {
    type Style: Default;

    fn active(&self, style: &Self::Style) -> Option<Appearance>;

    fn hovered(&self, style: &Self::Style) -> Option<Appearance>;
}
