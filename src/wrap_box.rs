use iced::keyboard;
use iced_native::{
    application::StyleSheet,
    event, layout, mouse, overlay,
    widget::{
        self, operation,
        scrollable::{Properties, RelativeOffset},
        tree, Id, Tree,
    },
    Alignment, Element, Layout, Length, Padding, Pixels, Point, Rectangle,
    Vector, Widget,
};

/// Container that distributes its contents both vertically and horizontaly
/// and also has a scollbar. Advantage over normal scrollbar combined with
/// row or column is that this more efficiently handles large amounts of
/// childern. In the first versions it may not support the horizontal part.
///
/// This is not finished and currently is the same as iced::widgets::Column
///
/// The code is hevily inspired by iced::widgets::Column and
/// iced::widgets::Scrollable.
pub struct WrapBox<'a, Message, Renderer: iced_native::Renderer>
where
    Renderer::Theme: StyleSheet,
{
    // reusing the scrollable structs
    id: Option<Id>,
    //spacing_x: f32,
    spacing_y: f32,
    padding: Padding,
    width: Length,
    height: Length,
    align_x: Alignment,
    //align_y: Alignment,
    vertical: Properties,
    horizontal: Option<Properties>,
    children: Vec<Element<'a, Message, Renderer>>,
    on_scroll: Option<Box<dyn Fn(Viewport) -> Message + 'a>>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer: iced_native::Renderer>
    WrapBox<'a, Message, Renderer>
where
    Renderer::Theme: StyleSheet,
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
            id: None,
            //spacing_x: 0.,
            spacing_y: 0.,
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            // max_height: f32::MAX,
            align_x: Alignment::Start,
            // align_y: Alignment::Start,
            vertical: Properties::default(),
            horizontal: None,
            children: childern,
            on_scroll: None,
            style: Default::default(),
        }
    }

    /// sets the [`Id`] of the [`WrapBox`]
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
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

    /// Sets the horizontal alignment of the [`WrapBox`]
    pub fn align_x(mut self, alignment: impl Into<Alignment>) -> Self {
        self.align_x = alignment.into();
        self
    }

    /// Sets the vertical scroll bar of the [`WrapBox`]
    pub fn vertical_scroll(mut self, properties: Properties) -> Self {
        self.vertical = properties;
        self
    }

    /// Sets the horizontal scroll bar of the [`WrapBox`]
    pub fn horizontal_scroll(mut self, properties: Properties) -> Self {
        self.horizontal = Some(properties);
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

    pub fn on_scroll(mut self, f: impl Fn(Viewport) -> Message + 'a) -> Self {
        self.on_scroll = Some(Box::new(f));
        self
    }

    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Renderer: iced_native::Renderer> Default
    for WrapBox<'a, Message, Renderer>
where
    Renderer::Theme: StyleSheet,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for WrapBox<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> widget::tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

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
        let limits = limits.width(self.width).height(self.height);

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
where
    Renderer::Theme: StyleSheet,
{
    fn from(value: WrapBox<'a, Message, Renderer>) -> Self {
        Self::new(value)
    }
}

/// The current [`Viewport`] of [`WrapBox`]
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    offset_x: Offset,
    offset_y: Offset,
    bounds: Rectangle,
    content_bounds: Rectangle,
}

impl Viewport {
    pub fn absolute_offset(&self) -> AbsoluteOffset {
        AbsoluteOffset {
            x: self
                .offset_x
                .absolute(self.bounds.width, self.content_bounds.width),
            y: self
                .offset_y
                .absolute(self.bounds.height, self.content_bounds.height),
        }
    }

    pub fn relative_offset(&self) -> RelativeOffset {
        let AbsoluteOffset { x, y } = self.absolute_offset();

        RelativeOffset {
            x: x / (self.content_bounds.width - self.bounds.width),
            y: y / (self.content_bounds.height - self.bounds.height),
        }
    }
}

/// The amount of offset in each direction of [`WrapBox`]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AbsoluteOffset {
    pub x: f32,
    pub y: f32,
}

/// The local state of [`WrapBox`]
#[derive(Debug, Clone, Copy)]
pub struct State {
    scroll_area_touched_at: Option<Point>,
    offset_y: Offset,
    y_scroller_grabbed_at: Option<f32>,
    offset_x: Offset,
    x_scroller_grabbed_at: Option<f32>,
    keyboard_modifiers: keyboard::Modifiers,
    last_notified: Option<Viewport>,
}

impl State {
    /// Creates a new [`State`] with the scrollbar(s) at the beginning.
    pub fn new() -> Self {
        State::default()
    }

    /// Apply a scrolling offset to the current [`State`], given the bounds of
    /// the [`WrapBox`] and its contents.
    pub fn scroll(
        &mut self,
        delta: Vector<f32>,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        if bounds.height < content_bounds.height {
            self.offset_y = Offset::Absolute(
                (self.offset_y.absolute(bounds.height, content_bounds.height)
                    - delta.y)
                    .clamp(0., content_bounds.height - bounds.height),
            );
        }

        if bounds.width < content_bounds.width {
            self.offset_x = Offset::Absolute(
                (self.offset_y.absolute(bounds.width, content_bounds.width)
                    - delta.x)
                    .clamp(0., content_bounds.width - bounds.height),
            );
        }
    }

    /// Scrolls the [`WrapBox`] to a relative amount along the x axis.
    ///
    /// `0` represents scrollbar at the beginning, while `1` represents scrollbar at
    /// the end.
    pub fn scroll_x_to(
        &mut self,
        percentage: f32,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        self.offset_x = Offset::Relative(percentage.clamp(0., 1.));
        self.unsnap(bounds, content_bounds);
    }

    /// Scrolls the [`WrapBox`] to a relative amount along the y axis.
    ///
    /// `0` represents scrollbar at the beginning, while `1` represents scrollbar at
    /// the end.
    pub fn scroll_y_to(
        &mut self,
        percentage: f32,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        self.offset_y = Offset::Relative(percentage.clamp(0., 1.));
        self.unsnap(bounds, content_bounds);
    }

    /// Snaps the scroll position to a [`RelativeOffset`].
    pub fn snap_to(&mut self, offset: RelativeOffset) {
        self.offset_x = Offset::Absolute(offset.x.max(0.));
        self.offset_y = Offset::Absolute(offset.y.max(0.));
    }

    /// Scroll to the provided [`AbsoluteOffset`].
    pub fn scroll_to(&mut self, offset: AbsoluteOffset) {
        self.offset_x = Offset::Absolute(offset.x.max(0.));
        self.offset_y = Offset::Absolute(offset.y.max(0.));
    }

    /// Unsnaps the current scroll position, if snapped, given the bounds of the
    /// [`WrapBox`] and its contents.
    pub fn unsnap(&mut self, bounds: Rectangle, content_bounds: Rectangle) {
        self.offset_x = Offset::Absolute(
            self.offset_x.absolute(bounds.width, content_bounds.width),
        );
        self.offset_y = Offset::Absolute(
            self.offset_y.absolute(bounds.height, content_bounds.height),
        );
    }

    /// Returns the scrolling offset of the [`State`], given the bounds of the
    /// [`WrapBox`] and its contents.
    pub fn offset(
        &self,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) -> Vector {
        Vector::new(
            self.offset_x.absolute(bounds.width, content_bounds.width),
            self.offset_y.absolute(bounds.height, content_bounds.height),
        )
    }

    pub fn scrllers_grabbed(&self) -> bool {
        self.x_scroller_grabbed_at.is_some()
            || self.y_scroller_grabbed_at.is_some()
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            scroll_area_touched_at: None,
            offset_y: Offset::Absolute(0.),
            y_scroller_grabbed_at: None,
            offset_x: Offset::Absolute(0.),
            x_scroller_grabbed_at: None,
            keyboard_modifiers: keyboard::Modifiers::default(),
            last_notified: None,
        }
    }
}

impl operation::Scrollable for State {
    fn snap_to(&mut self, offset: RelativeOffset) {
        State::snap_to(self, offset);
    }
}

#[derive(Debug, Clone, Copy)]
enum Offset {
    Absolute(f32),
    Relative(f32),
}

impl Offset {
    fn absolute(self, viewport: f32, content: f32) -> f32 {
        match self {
            Offset::Absolute(absolute) => {
                absolute.min((content - viewport).max(0.))
            }
            Offset::Relative(percentage) => {
                ((content - viewport) * percentage).max(0.)
            }
        }
    }
}

pub fn wrap_box<Message, Renderer: iced_native::Renderer>(
    childern: Vec<Element<'_, Message, Renderer>>,
) -> WrapBox<'_, Message, Renderer>
where
    Renderer::Theme: StyleSheet,
{
    WrapBox::with_childern(childern)
}
