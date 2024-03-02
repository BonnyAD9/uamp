use iced_core::{
    layout::{Limits, Node},
    mouse::Cursor,
    renderer::{Quad, Style},
    widget::Tree,
    Background, Color, Element, Layout, Length, Rectangle, Vector, Widget,
};

use super::{limit_size, sides::Sides, NO_SHADOW};

pub struct Border<'a, Message, Theme, Renderer>
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

impl<'a, Message, Theme, Renderer> Border<'a, Message, Theme, Renderer>
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
    for Border<'a, Message, Theme, Renderer>
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
        let bounds = layout.bounds();

        let thickness = theme.border_thickness(&self.style);
        let radius = theme.border_radius(&self.style);
        let border_radius = theme.border_border_radius(&self.style);
        let color = theme.border_color(&self.style);

        // Left border
        if thickness.left != 0. {
            let bounds = Rectangle {
                x: bounds.x - thickness.left,
                y: bounds.y + radius.top,
                width: thickness.left,
                height: bounds.height - radius.top - radius.left,
            };

            let quad = Quad {
                bounds,
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.,
                    radius: border_radius.left.into(),
                },
                shadow: NO_SHADOW,
            };

            renderer.fill_quad(quad, color.left);
        }

        // Top border
        if thickness.top != 0. {
            let bounds = Rectangle {
                x: bounds.x + radius.top,
                y: bounds.y - thickness.top,
                width: bounds.width - radius.top - radius.right,
                height: thickness.top,
            };

            let quad = Quad {
                bounds,
                border: iced::Border {
                    radius: border_radius.top.into(),
                    width: 0.,
                    color: Color::TRANSPARENT,
                },
                shadow: NO_SHADOW,
            };

            renderer.fill_quad(quad, color.top);
        }

        // Right border
        if thickness.right != 0. {
            let bounds = Rectangle {
                x: bounds.x + bounds.width,
                y: bounds.y + radius.right,
                width: thickness.right,
                height: bounds.height - radius.right - radius.bottom,
            };

            let quad = Quad {
                bounds,
                border: iced::Border {
                    radius: border_radius.left.into(),
                    width: 0.,
                    color: Color::TRANSPARENT,
                },
                shadow: NO_SHADOW,
            };

            renderer.fill_quad(quad, color.right);
        }

        // Bottom border
        if thickness.bottom != 0. {
            let bounds = Rectangle {
                x: bounds.x + radius.bottom_left(),
                y: bounds.y + bounds.height,
                width: bounds.width
                    - radius.bottom_left()
                    - radius.bottom_right(),
                height: thickness.bottom,
            };

            let quad = Quad {
                bounds,
                border: iced::Border {
                    radius: border_radius.bottom.into(),
                    width: 0.,
                    color: Color::TRANSPARENT,
                },
                shadow: NO_SHADOW,
            };

            renderer.fill_quad(quad, color.bottom);
        }

        // The other features are not supported yet

        let quad = Quad {
            bounds,
            border: iced::Border {
                radius: radius.into(),
                width: 0.,
                color: Color::TRANSPARENT,
            },
            shadow: NO_SHADOW,
        };

        let bg = theme.background(&self.style);
        renderer.fill_quad(quad, bg);

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

impl<'a, Message, Theme, Renderer> From<Border<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
    Theme: StyleSheet + 'a,
    Message: 'a,
{
    fn from(value: Border<'a, Message, Theme, Renderer>) -> Self {
        Self::new(value)
    }
}

pub trait StyleSheet {
    type Style: Default;

    fn background(&self, style: &Self::Style) -> Background;

    /// Thickness of the sides of the border
    fn border_thickness(&self, style: &Self::Style) -> Sides<f32>;

    /// Thickness of the border in corners
    fn corner_thickness(&self, style: &Self::Style) -> Sides<f32> {
        self.border_thickness(style)
    }

    /// Border radius of the corners
    fn border_radius(&self, style: &Self::Style) -> Sides<f32>;

    /// Returns the radius of the borders
    fn border_border_radius(&self, style: &Self::Style) -> Sides<Sides<f32>> {
        _ = style;
        let s: Sides<f32> = 0.into();
        s.into()
    }

    /// Returns the radius of the borders
    fn corner_border_radius(&self, style: &Self::Style) -> Sides<Sides<f32>> {
        _ = style;
        let s: Sides<f32> = 0.into();
        s.into()
    }

    /// Returns the color of the borders
    fn border_color(&self, style: &Self::Style) -> Sides<Background>;

    /// Returns the color of the corner borders
    fn corner_color(&self, style: &Self::Style) -> Sides<Color>;
}
