use std::vec;

use iced_native::{
    event,
    layout::{self, Node, Limits},
    mouse::{self, ScrollDelta},
    overlay::Group,
    widget::{self, tree, Tree},
    Alignment, Element, Layout, Length, Padding, Pixels, Point, Rectangle,
    Size, Vector, Widget,
};

use self::ItemDirection::{
    BottomToTop, LeftToRight, RightToLeft, TopToBottom,
};

/// Container that distributes its contents both vertically and horizontaly
/// and also has a scollbar. Advantage over normal scrollbar combined with
/// row or column is that this more efficiently handles large amounts of
/// childern. In the first versions it may not support the horizontal part.
///
/// This is not finished and currently is the same as iced::widgets::Column
///
/// The code is hevily inspired by iced::widgets::Column
pub struct WrapBox<'a, Message, Renderer: iced_native::Renderer> {
    spacing_x: f32,
    spacing_y: f32,
    padding: Padding,
    width: Length,
    height: Length,
    max_width: f32,
    max_height: f32,
    item_width: f32,
    item_height: f32,
    max_wrap: u32,
    min_wrap: u32,
    wrap_jump: u32,
    primary_direction: ItemDirection,
    secondary_direction: ItemDirection,
    children: Vec<Element<'a, Message, Renderer>>,
    state: Option<State>,
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
            spacing_x: 0.,
            spacing_y: 0.,
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: f32::MAX,
            max_height: f32::MAX,
            item_width: 0.,
            item_height: 0.,
            max_wrap: u32::MAX,
            min_wrap: 1,
            wrap_jump: 1,
            primary_direction: ItemDirection::LeftToRight,
            secondary_direction: ItemDirection::TopToBottom,
            children: childern,
            state: None,
        }
    }

    /// Sets the horizontal spacing between elements
    pub fn spacing_x(mut self, amount: impl Into<Pixels>) -> Self {
        self.spacing_x = amount.into().0;
        self
    }

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

    /// Sets the maximum width of the [`WrapBox`]
    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.max_width = max_width.into().0;
        self
    }

    /// Sets the maximum height f the [`WrapBox`]
    pub fn max_height(mut self, max_height: impl Into<Pixels>) -> Self {
        self.max_height = max_height.into().0;
        self
    }

    /// Sets fixed width of the items, 0 means that each item may determine
    /// its width by itself, use this when possible, because this will optimize
    /// all the interaction with the [`WrapBox`]
    pub fn item_width(mut self, item_width: impl Into<Pixels>) -> Self {
        self.item_width = item_width.into().0;
        self
    }

    /// Sets fixed height of the items, 0 means that each item may determine
    /// its height by itself, use this when possible, because this will
    /// optimize all the interaction with the [`WrapBox`]
    pub fn item_height(mut self, item_height: impl Into<Pixels>) -> Self {
        self.item_height = item_height.into().0;
        self
    }

    /// Sets the maximum items in the axis given by orientation before wrapping
    pub fn max_wrap(mut self, wrap_count: u32) -> Self {
        if wrap_count == 0 {
            self.max_wrap = 1;
        } else {
            self.max_wrap = wrap_count;
        }
        self
    }

    /// Sets the minimum items in the axis given by orientation before wrapping
    pub fn min_wrap(mut self, wrap_count: u32) -> Self {
        if wrap_count == 0 {
            self.min_wrap = 1;
        } else {
            self.min_wrap = wrap_count;
        }
        self
    }

    /// Sets both the min_wrap and max_wrap to the given value
    pub fn wrap_count(self, wrap_count: u32) -> Self {
        self.max_wrap(wrap_count).min_wrap(wrap_count)
    }

    /// Sets the wrap jump, number of items on the axis given by the
    /// orientation will be multiple of the wrap jump
    pub fn wrap_jump(mut self, wrap_jump: u32) -> Self {
        if wrap_jump == 0 {
            self.wrap_jump = 1;
        } else {
            self.wrap_jump = wrap_jump;
        }
        self
    }

    /// Sets the primary direction, if the secondary direction is in conflict
    /// the secundary direction is adjusted.
    ///
    /// #### Adjustments:
    /// - LeftToRight <-> TopToBottom
    /// - RightToLeft <-> BottomToTop
    pub fn primary_direction(mut self, direction: ItemDirection) -> Self {
        self.primary_direction = direction;
        match (direction, self.secondary_direction) {
            (LeftToRight | RightToLeft, LeftToRight) => {
                self.secondary_direction = TopToBottom;
            }
            (LeftToRight | RightToLeft, RightToLeft) => {
                self.secondary_direction = BottomToTop;
            }
            (TopToBottom | BottomToTop, TopToBottom) => {
                self.secondary_direction = LeftToRight;
            }
            (TopToBottom | BottomToTop, BottomToTop) => {
                self.secondary_direction = RightToLeft;
            }
            _ => {}
        }
        self
    }

    /// Sets the secondary direction, if the primary direction is in conflict
    /// the primary direction is adjusted.
    ///
    /// #### Adjustments:
    /// - LeftToRight <-> TopToBottom
    /// - RightToLeft <-> BottomToTop
    pub fn secondary_direction(mut self, direction: ItemDirection) -> Self {
        self.secondary_direction = direction;
        match (direction, self.primary_direction) {
            (LeftToRight | RightToLeft, LeftToRight) => {
                self.primary_direction = TopToBottom;
            }
            (LeftToRight | RightToLeft, RightToLeft) => {
                self.primary_direction = BottomToTop;
            }
            (TopToBottom | BottomToTop, TopToBottom) => {
                self.primary_direction = LeftToRight;
            }
            (TopToBottom | BottomToTop, BottomToTop) => {
                self.primary_direction = RightToLeft;
            }
            _ => {}
        }
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

impl<'a, Message: 'a, Renderer> Widget<Message, Renderer>
    for WrapBox<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer + 'a,
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
        let limits = limits
            .max_width(self.max_width)
            .max_height(self.max_height)
            .width(self.width)
            .height(self.height);

        // TODO: Properly handle the limits, don't just use max

        // skip the layout if it cannot be calculated efficiently
        // the nearest event will allow and trigger efficient layout
        if self.state.is_none() {
            return Node::new(limits.max());
        }

        let (first, last) = self.visible_pos(limits.max());

        let child_limits = layout::Limits::new(
            Size::new(limits.min().width, 0.),
            Size::new(limits.max().width, f32::MAX),
        );

        let node = layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            &child_limits,
            self.padding,
            self.spacing_y,
            Alignment::Start,
            &self.children[first..=last],
        );

        Node::with_children(limits.max(), vec![node])
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: iced_native::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        let child = match layout.children().next() {
            Some(c) => c,
            None => return
        };
        let state = tree.state.downcast_ref();

        operation.container(None, &mut |operation| {
            self.visible_state(layout.bounds().size(), *state)
                .zip(child.children())
                .for_each(|((child, i), layout)| {
                    child.as_widget().operate(
                        &mut tree.children[i],
                        layout,
                        renderer,
                        operation,
                    )
                })
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced_native::Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn iced_native::Clipboard,
        shell: &mut iced_native::Shell<'_, Message>,
    ) -> iced_native::event::Status {
        if matches!(event, iced_native::Event::Mouse(_)) && !layout.bounds().contains(cursor_position) {
            return iced_native::event::Status::Ignored
        }

        let state = tree.state.downcast_mut::<State>();
        let size = layout.bounds().size();
        let item_space = self.item_height + self.spacing_y;

        let (first_o, last_o) = self.visible_pos_state(size, *state);

        if let iced_native::Event::Mouse(mouse::Event::WheelScrolled {
            delta,
        }) = event
        {
            match delta {
                ScrollDelta::Lines { x: _, y } => {
                    state.offset_y -= y * item_space;
                }
                ScrollDelta::Pixels { x: _, y } => {
                    state.offset_y -= y;
                }
            }
        }
        state.offset_y = state
            .offset_y
            .min(item_space * self.children.len() as f32 - size.height)
            .max(0.);

        let (first, last) = self.visible_pos_state(size, *state);
        if first_o != first || last_o != last {
            shell.invalidate_layout();
        }

        self.state = Some(*state);

        let node; // just owner of potential memory
        let child = match layout.children().next() {
            Some(c) => c,
            None => {
                // when the layout is not available, calculate temporary layout
                // so that there is no dropped event
                shell.invalidate_layout();
                node = self.layout(renderer, &Limits::new(Size::new(0., 0.), size));
                Layout::new(node.children().iter().next().unwrap())
            }
        };

        self.visible_state_mut(size, *state)
            .zip(child.children())
            .map(|((child, i), layout)| {
                child.as_widget_mut().on_event(
                    &mut tree.children[i],
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
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if !layout.bounds().contains(cursor_position) {
            return mouse::Interaction::Idle
        }
        let child = match layout.children().next() {
            Some(c) => c,
            None => return mouse::Interaction::Idle
        };

        let state = tree.state.downcast_ref();

        self.visible_state(layout.bounds().size(), *state)
            .zip(child.children())
            .map(|((child, i), layout)| {
                child.as_widget().mouse_interaction(
                    &tree.children[i],
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
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced_native::Renderer>::Theme,
        style: &iced_native::renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let child = match layout.children().next() {
            Some(c) => c,
            None => return
        };

        let bounds = layout.bounds();

        let state = tree.state.downcast_ref::<State>();
        let item_space = self.item_height + self.spacing_y;

        let (first, _) = self.visible_pos_state(bounds.size(), *state);
        let offset =
            Vector::new(0., first as f32 * item_space - state.offset_y);

        let mouse_pos = if layout.bounds().contains(cursor_position) {
            cursor_position - offset
        } else {
            // don't count with the mous if it is outside
            Point::new(f32::INFINITY, f32::INFINITY)
        };
        let child_viewport = Rectangle {
            x: bounds.x - state.offset_x,
            y: bounds.y - state.offset_y,
            ..bounds
        };

        renderer.with_layer(bounds, |renderer| {
            renderer.with_translation(offset, |renderer| {
                for ((child, i), layout) in self
                    .visible_state(layout.bounds().size(), *state)
                    .zip(child.children())
                {
                    child.as_widget().draw(
                        &tree.children[i],
                        renderer,
                        theme,
                        style,
                        layout,
                        mouse_pos,
                        &child_viewport,
                    );
                }
            })
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<iced_native::overlay::Element<'b, Message, Renderer>> {
        let child = match layout.children().next() {
            Some(c) => c,
            None => return None
        };

        let state = tree.state.downcast_ref::<State>();

        let (first, last) =
            self.visible_pos_state(layout.bounds().size(), *state);

        let children = self
            .visible_state_mut(layout.bounds().size(), *state)
            .zip(&mut tree.children[first..=last])
            .zip(child.children())
            .filter_map(|(((child, _), state), layout)| {
                child.as_widget_mut().overlay(state, layout, renderer)
            })
            .collect::<Vec<_>>();

        (!children.is_empty())
            .then(|| Group::with_children(children).overlay())
    }
}

impl<'a, Message: 'a, Renderer: iced_native::Renderer + 'a>
    From<WrapBox<'a, Message, Renderer>> for Element<'a, Message, Renderer>
{
    fn from(value: WrapBox<'a, Message, Renderer>) -> Self {
        Self::new(value)
    }
}

impl<'a, Message: 'a, Renderer: iced_native::Renderer + 'a>
    WrapBox<'a, Message, Renderer>
{
    #[inline]
    fn visible_pos_state(
        &self,
        view_size: Size,
        state: State,
    ) -> (usize, usize) {
        if self.item_height == 0. {
            (0, self.children.len() - 1)
        } else {
            let item_space = self.item_height + self.spacing_y;
            (
                (state.offset_y / item_space).max(0.) as usize,
                (((state.offset_y + view_size.height) / item_space) as usize)
                    .min(self.children.len() - 1),
            )
        }
    }

    #[inline]
    fn visible_pos(&self, view_size: Size) -> (usize, usize) {
        if let Some(state) = self.state {
            self.visible_pos_state(view_size, state)
        } else {
            (0, self.children.len() - 1)
        }
    }

    #[inline]
    fn visible(
        &'a self,
        view_size: Size,
    ) -> impl Iterator<Item = (&Element<'a, Message, Renderer>, usize)> {
        let (first, last) = self.visible_pos(view_size);
        self.children[first..=last]
            .iter()
            .enumerate()
            .map(move |(i, c)| (c, i + first))
    }

    #[inline]
    fn visible_state(
        &'a self,
        view_size: Size,
        state: State,
    ) -> impl Iterator<Item = (&Element<'a, Message, Renderer>, usize)> {
        let (first, last) = self.visible_pos_state(view_size, state);
        self.children[first..=last]
            .iter()
            .enumerate()
            .map(move |(i, c)| (c, i + first))
    }

    #[inline]
    fn visible_state_mut(
        &mut self,
        view_size: Size,
        state: State,
    ) -> impl Iterator<Item = (&mut Element<'a, Message, Renderer>, usize)>
    {
        let (first, last) = self.visible_pos_state(view_size, state);
        self.children[first..=last]
            .iter_mut()
            .enumerate()
            .map(move |(i, c)| (c, i + first))
    }

    #[inline]
    fn offset(&self) -> Option<(f32, f32)> {
        if let Some(state) = self.state {
            if self.item_height == 0. {
                None
            } else {
                Some((state.offset_x, state.offset_y))
            }
        } else {
            None
        }
    }
}

pub fn wrap_box<Message, Renderer: iced_native::Renderer>(
    childern: Vec<Element<'_, Message, Renderer>>,
) -> WrapBox<'_, Message, Renderer> {
    WrapBox::with_childern(childern)
}

#[derive(Copy, Clone)]
pub enum ItemDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

#[derive(Copy, Clone)]
struct State {
    /// Absolute offset on the x axis
    offset_x: f32,
    /// Absolute offset on the y axis
    offset_y: f32,
}

impl State {
    fn new() -> Self {
        Self {
            offset_x: 0.,
            offset_y: 0.,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
