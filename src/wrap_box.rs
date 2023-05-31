use std::vec;

use iced::Renderer;
use iced_native::{
    event, layout::{self, Node},
    mouse::{self, ScrollDelta},
    overlay,
    widget::{self, pane_grid::Direction, tree, Tree},
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

        let VisibleChildernIterator {
            wrap_box: _,
            i: first_item,
            max_i: last_item,
        } = self.visible(limits.max());

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
            &self.children[first_item..=last_item],
        );

        Node::with_children(limits.max(), vec![
        if let Some((x, y)) = self.offset() {
            node.translate(Vector::new(x, y))
        } else {
            node
        }])
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: iced_native::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        let state = tree.state.downcast_ref();

        operation.container(None, &mut |operation| {
            self.visible_state(layout.bounds().size(), *state)
                .zip(layout.children().next().unwrap().children())
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
        let state = tree.state.downcast_mut::<State>();

        // BUG: incorrect scroll bounds
        // BUG: incorrect offset after changing height of window
        if let iced_native::Event::Mouse(mouse::Event::WheelScrolled {
            delta,
        }) = event
        {
            let item_space = self.item_height + self.spacing_y;
            match delta {
                ScrollDelta::Lines { x: _, y } => {
                    state.offset_y -= y * item_space;
                }
                ScrollDelta::Pixels { x: _, y } => {
                    state.offset_y -= y;
                }
            }
            state.offset_y = state.offset_y.max(0.).min(item_space * self.children.len() as f32 - layout.bounds().size().height);
        }

        self.state = Some(*state);

        self.visible_state_mut(layout.bounds().size(), *state)
            .zip(layout.children().next().unwrap().children())
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
        let state = tree.state.downcast_ref();

        self.visible_state(layout.bounds().size(), *state)
            .zip(layout.children().next().unwrap().children())
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
        tree: &iced_native::widget::Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced_native::Renderer>::Theme,
        style: &iced_native::renderer::Style,
        layout: iced_native::Layout<'_>,
        cursor_position: iced_native::Point,
        viewport: &iced_native::Rectangle,
    ) {
        let state = tree.state.downcast_ref();

        for ((child, i), layout) in self
            .visible_state(layout.bounds().size(), *state)
            .zip(layout.children().next().unwrap().children())
        {
            child.as_widget().draw(
                &tree.children[i],
                renderer,
                theme,
                style,
                layout,
                cursor_position,
                viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<iced_native::overlay::Element<'b, Message, Renderer>> {
        //overlay::from_children(&mut self.children, state, layout, renderer)
        None
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
    fn visible(
        &'a self,
        view_size: Size,
    ) -> VisibleChildernIterator<'a, Message, Renderer> {
        if let Some(state) = self.state {
            VisibleChildernIterator::new(self, state, view_size)
        } else {
            VisibleChildernIterator {
                wrap_box: self,
                i: 0,
                max_i: self.children.len() - 1,
            }
        }
    }

    fn visible_state(
        &'a self,
        view_size: Size,
        state: State,
    ) -> VisibleChildernIterator<'a, Message, Renderer> {
        VisibleChildernIterator::new(self, state, view_size)
    }

    fn visible_state_mut(
        &mut self,
        view_size: Size,
        state: State,
    ) -> impl Iterator<Item = (&mut Element<'a, Message, Renderer>, usize)>
    {
        let VisibleChildernIterator {
            wrap_box: _,
            i: first,
            max_i: last,
        } = self.visible_state(view_size, state);
        self.children[first..=last]
            .iter_mut()
            .enumerate()
            .map(move |(i, c)| (c, i + first))
    }

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

struct VisibleChildernIterator<'a, Message, Renderer: iced_native::Renderer> {
    wrap_box: &'a WrapBox<'a, Message, Renderer>,
    i: usize,
    max_i: usize,
}

impl<'a, Message, Renderer: iced_native::Renderer>
    VisibleChildernIterator<'a, Message, Renderer>
{
    fn new(
        wrap_box: &'a WrapBox<'a, Message, Renderer>,
        state: State,
        view_size: Size,
    ) -> Self {
        if wrap_box.item_height == 0. {
            VisibleChildernIterator {
                wrap_box,
                i: 0,
                max_i: wrap_box.children.len() - 1,
            }
        } else {
            let item_space = wrap_box.item_height + wrap_box.spacing_y;
            VisibleChildernIterator {
                wrap_box,
                i: (state.offset_y / item_space).max(0.) as usize,
                max_i: (((state.offset_y + view_size.height) / item_space)
                    as usize)
                    .min(wrap_box.children.len() - 1),
            }
        }
    }
}

impl<'a, Message, Renderer: iced_native::Renderer> Iterator
    for VisibleChildernIterator<'a, Message, Renderer>
{
    type Item = (&'a Element<'a, Message, Renderer>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i > self.max_i {
            None
        } else {
            let res = Some((&self.wrap_box.children[self.i], self.i));
            self.i += 1;
            res
        }
    }
}
