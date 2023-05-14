use std::borrow::Borrow;

use iced_native::{
    event, layout, mouse, overlay,
    widget::{self, Tree},
    Alignment, Element, Layout, Length, Padding, Pixels, Point, Rectangle,
    Widget,
};

/// Container that distributes its contents both vertically and horizontaly
/// and also has a scollbar. Advantage over normal scrollbar combined with
/// row or column is that this more efficiently handles large amounts of
/// childern. In the first versions it may not support the horizontal part.
///
/// The code is hevily inspired by iced::widgets::Column
pub struct WrapBox<'a, Message, Renderer: iced_native::Renderer> {
    //spacing_x: f32,
    spacing_y: f32,
    padding: Padding,
    width: Length,
    height: Length,
    max_width: f32,
    //max_height: f32,
    align_x: Alignment,
    //align_y: Alignment,
    children: Vec<Element<'a, Message, Renderer>>,
}

impl<'a, Message, Renderer: iced_native::Renderer>
    WrapBox<'a, Message, Renderer>
{
    /// creates empty [`WrapBox`]
    pub fn new() -> Self {
        Self::with_childern(Vec::new())
    }

    /// creates a [`WrapBox`] with the given elements
    pub fn with_childern(
        childern: Vec<Element<'a, Message, Renderer>>,
    ) -> Self {
        WrapBox {
            //spacing_x: 0.,
            spacing_y: 0.,
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: f32::MAX,
            // max_height: f32::MAX,
            align_x: Alignment::Start,
            // align_y: Alignment::Start,
            children: childern,
        }
    }

    /// Sets the horizontal spacing between elements
    /*pub fn spacing_x(mut self, amount: impl Into<Pixels>) -> Self {
        self.spacing_x = amount.into().0;
        self
    }*/

    /// Sets the vertical spacing between elements
    pub fn spacing_y(mut self, amount: impl Into<Pixels>) -> Self {
        self.spacing_y = amount.into().0;
        self
    }

    /// Sets the [`Padding`] of the [`WrapBox`]
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [`WrapBox`]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`WrapBox`]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /*pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }*/

    /// Sets the maximum width of the [`WrapBox`]
    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.max_width = max_width.into().0;
        self
    }

    /// Sets the horizontal alignment of the [`WrapBox`]
    pub fn align_x(mut self, alignment: impl Into<Alignment>) -> Self {
        self.align_x = alignment.into();
        self
    }

    /// Adds element to the [`WrapBox`]
    pub fn push(
        mut self,
        child: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        self.children.push(child.into());
        self
    }
}

impl<'a, Message, Renderer: iced_native::Renderer> Default
    for WrapBox<'a, Message, Renderer>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for WrapBox<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn width(&self) -> iced_native::Length {
        self.width
    }

    fn height(&self) -> iced_native::Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &iced_native::layout::Limits,
    ) -> iced_native::layout::Node {
        let limits = limits
            .max_width(self.max_width)
            .width(self.width)
            .height(self.height);

        layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            &limits,
            self.padding,
            self.spacing_y,
            self.align_x,
            &self.children,
        )
    }

    fn operate(
        &self,
        state: &mut Tree,
        layout: iced_native::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        operation.container(None, &mut |operation| {
            self.children
                .iter()
                .zip(&mut state.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation)
                })
        });
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: iced_native::Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn iced_native::Clipboard,
        shell: &mut iced_native::Shell<'_, Message>,
    ) -> iced_native::event::Status {
        self.children
            .iter_mut()
            .zip(&mut state.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    shell,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&state.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget().mouse_interaction(
                    state,
                    layout,
                    cursor_position,
                    viewport,
                    renderer,
                )
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        state: &iced_native::widget::Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced_native::Renderer>::Theme,
        style: &iced_native::renderer::Style,
        layout: iced_native::Layout<'_>,
        cursor_position: iced_native::Point,
        viewport: &iced_native::Rectangle,
    ) {
        for ((child, state), layout) in self
            .children
            .iter()
            .zip(&state.children)
            .zip(layout.children())
        {
            child.as_widget().draw(
                state,
                renderer,
                theme,
                style,
                layout,
                cursor_position,
                viewport,
            )
        }
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<iced_native::overlay::Element<'b, Message, Renderer>> {
        overlay::from_children(&mut self.children, state, layout, renderer)
    }
}

impl<'a, Message: 'a, Renderer: iced_native::Renderer + 'a>
    From<WrapBox<'a, Message, Renderer>> for Element<'a, Message, Renderer>
{
    fn from(value: WrapBox<'a, Message, Renderer>) -> Self {
        Self::new(value)
    }
}

pub fn wrap_box<Message, Renderer: iced_native::Renderer>(
    childern: Vec<Element<'_, Message, Renderer>>,
) -> WrapBox<'_, Message, Renderer> {
    WrapBox::with_childern(childern)
}
