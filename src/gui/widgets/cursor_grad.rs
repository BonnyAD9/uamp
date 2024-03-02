use iced_core::{
    gradient::Linear,
    layout::{Limits, Node},
    mouse::Cursor,
    renderer::{Quad, Style},
    widget::Tree,
    Background, Color, Degrees, Element, Gradient, Layout, Length, Rectangle,
    Vector, Widget,
};

use super::{limit_size, sides::Sides, NO_SHADOW};

pub struct CursorGrad<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
    Theme: StyleSheet,
{
    width: Length,
    height: Length,
    padding: Sides<f32>,
    child: Element<'a, Message, Theme, Renderer>,
    style: <Theme as StyleSheet>::Style,
}

impl<'a, Message, Theme, Renderer> CursorGrad<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
    Theme: StyleSheet,
{
    /// Creates new border
    pub fn new(child: Element<'a, Message, Theme, Renderer>) -> Self {
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
        style: <Theme as StyleSheet>::Style,
    ) -> Self {
        self.style = style;
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for CursorGrad<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
    Theme: StyleSheet,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.child)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.child])
    }

    fn size(&self) -> iced::Size<Length> {
        iced::Size { width: self.width, height: self.height }
    }

    fn layout(&self, state: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let lim = limits.width(self.width).height(self.height);

        let child_limits = lim.shrink(self.padding);

        let child = self
            .child
            .as_widget()
            .layout(&mut state.children[0], renderer, &child_limits)
            .translate(Vector::new(self.padding.left, self.padding.top));

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
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        self.child.as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );

        let bounds = layout.bounds();

        let ap = if cursor.is_over(bounds) {
            theme.hovered(&self.style)
        } else {
            theme.active(&self.style)
        };

        let ap = if let Some(ap) = ap {
            ap
        } else {
            return;
        };

        let pos = if let Some(p) = cursor.position() {
            p
        } else {
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
            border: iced::Border {
                radius: ap.border_radius.into(),
                width: 0.,
                color: Color::TRANSPARENT,
            },
            shadow: NO_SHADOW,
        };

        renderer.fill_quad(quad, Background::Gradient(Gradient::Linear(grad)));
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<iced_core::overlay::Element<'b, Message, Theme, Renderer>> {
        self.child.as_widget_mut().overlay(
            &mut state.children[0],
            layout.children().next().unwrap(),
            renderer,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<CursorGrad<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
    Theme: StyleSheet + 'a,
    Message: 'a,
{
    fn from(value: CursorGrad<'a, Message, Theme, Renderer>) -> Self {
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
