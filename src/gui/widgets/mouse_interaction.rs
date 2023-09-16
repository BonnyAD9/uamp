use iced_core::{
    layout::{Limits, Node},
    mouse::Cursor,
    renderer::Style,
    widget::Tree,
    Element, Layout, Length, Rectangle, Vector, Widget,
};

use super::{limit_size, sides::Sides};

pub struct MouseInteraction<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
    Message: Clone,
{
    width: Length,
    height: Length,
    padding: Sides<f32>,
    child: Element<'a, Message, Renderer>,
    mouse_over: Option<bool>,
    mouse_enter: Option<Message>,
}

impl<'a, Message, Renderer> MouseInteraction<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
    Message: Clone,
{
    /// Creates new border
    pub fn new(child: Element<'a, Message, Renderer>) -> Self {
        Self {
            width: Length::Shrink,
            height: Length::Shrink,
            padding: 0.into(),
            child,
            mouse_over: None,
            mouse_enter: None,
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

    pub fn on_mouse_enter(mut self, msg: Message) -> Self {
        self.mouse_enter = Some(msg);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for MouseInteraction<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
    Message: Clone,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.child)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.child])
    }

    fn width(&self) -> Length {
        if self.width == Length::Shrink
            && self.child.as_widget().width() == Length::Fill
        {
            Length::Fill
        } else {
            self.width
        }
    }

    fn height(&self) -> Length {
        if self.height == Length::Shrink
            && self.child.as_widget().height() == Length::Fill
        {
            Length::Fill
        } else {
            self.height
        }
    }

    fn layout(&self, renderer: &Renderer, limits: &Limits) -> Node {
        let lim = limits.width(self.width).height(self.height);

        let child_limits = lim.pad(self.padding.into());

        let child = self
            .child
            .as_widget()
            .layout(renderer, &child_limits)
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
        let mouse_over = cursor.is_over(layout.bounds());

        if let Some(mo) = self.mouse_over {
            if mouse_over && !mo {
                if let Some(msg) = self.mouse_enter.clone() {
                    shell.publish(msg);
                }
            }
        }

        self.mouse_over = Some(mouse_over);

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

impl<'a, Message, Renderer> From<MouseInteraction<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
    Message: Clone,
    Message: 'a,
{
    fn from(value: MouseInteraction<'a, Message, Renderer>) -> Self {
        Self::new(value)
    }
}
