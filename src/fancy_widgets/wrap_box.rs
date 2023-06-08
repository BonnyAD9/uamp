use std::vec;

use iced_native::{
    color, event,
    layout::{Limits, Node},
    mouse::{self, Button, ScrollDelta},
    overlay::Group,
    renderer::{BorderRadius, Quad},
    widget::{self, tree, Tree},
    Background, Color, Element, Layout, Length, Padding, Pixels, Point,
    Rectangle, Size, Theme, Vector, Widget,
};
use iced_native::{renderer, text};

use self::ItemDirection::{
    BottomToTop, LeftToRight, RightToLeft, TopToBottom,
};

pub const DEFAULT_SCROLL_SPEED: f32 = 60.;
pub const DEFAULT_SCROLLBAR_WIDTH: f32 = 20.;
pub const DEFAULT_SCROLLBAR_BUTTON_HEIGHT: f32 = 20.;
pub const DEFAULT_MIN_THUMB_SIZE: f32 = 20.;

/// Container that distributes its contents both vertically and horizontaly
/// and also has a scollbar. Advantage over normal scrollbar combined with
/// row or column is that this more efficiently handles large amounts of
/// childern. In the first versions it may not support the horizontal part.
///
/// This is not finished and currently supports only vertical scrolling
pub struct WrapBox<'a, Message, Renderer: text::Renderer>
where
    Renderer::Theme: WrapBoxStyleSheet,
{
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
    line_scroll: f32,
    scrollbar_width: f32,
    scrollbar_button_height: f32,
    min_thumb_size: f32,
    primary_direction: ItemDirection,
    secondary_direction: ItemDirection,
    primary_scrollbar: Behaviour,
    secondary_scrollbar: Behaviour,
    children: Vec<Element<'a, Message, Renderer>>,
    state: Option<State>,
    style: <Renderer::Theme as WrapBoxStyleSheet>::Style,
}

impl<'a, Message, Renderer: text::Renderer> WrapBox<'a, Message, Renderer>
where
    Renderer::Theme: WrapBoxStyleSheet,
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
            width: Length::Fill,
            height: Length::Fill,
            max_width: f32::MAX,
            max_height: f32::MAX,
            item_width: 0.,
            item_height: 0.,
            max_wrap: u32::MAX,
            min_wrap: 1,
            line_scroll: 0.,
            scrollbar_width: DEFAULT_SCROLLBAR_WIDTH,
            scrollbar_button_height: DEFAULT_SCROLLBAR_BUTTON_HEIGHT,
            min_thumb_size: DEFAULT_MIN_THUMB_SIZE,
            primary_direction: ItemDirection::LeftToRight,
            secondary_direction: ItemDirection::TopToBottom,
            primary_scrollbar: Behaviour::Enabled,
            secondary_scrollbar: Behaviour::Disabled,
            children: childern,
            state: None,
            style: Default::default(),
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
    /// all the interaction with the [`WrapBox`], negative value will signify
    /// that the width is max, and it will not use the optimization
    pub fn item_width(mut self, item_width: impl Into<Pixels>) -> Self {
        self.item_width = item_width.into().0;
        self
    }

    /// Sets fixed height of the items, 0 means that each item may determine
    /// its height by itself, use this when possible, because this will
    /// optimize all the interaction with the [`WrapBox`], negative value will
    /// signify that the height is max, and it will not use the optimization
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

    /// Sets the amount of pixels to scroll for each line, zero is the item
    /// height (if set) otherwise it is [`DEFAULT_SCROLL_SPEED`]
    pub fn line_scroll(mut self, scroll_amount: impl Into<Pixels>) -> Self {
        self.line_scroll = scroll_amount.into().0;
        self
    }

    // sets the width of the scrollbar (this is the height of the horizontal
    // scrollbar)
    pub fn scrollbar_width(mut self, width: impl Into<Pixels>) -> Self {
        self.scrollbar_width = width.into().0;
        self
    }

    // sets the height of the scrollbar buttons (this is width of the buttons
    // on the horizontal scrollbar)
    pub fn scrollbar_button_height(
        mut self,
        height: impl Into<Pixels>,
    ) -> Self {
        self.scrollbar_button_height = height.into().0;
        self
    }

    // sets the width of the scrollbar (this is the height of the horizontal
    // scrollbar)
    pub fn min_thumb_size(mut self, height: impl Into<Pixels>) -> Self {
        self.min_thumb_size = height.into().0;
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

    /// enable or disable the primary scrollbar of the [`WrapBox`]
    pub fn primary_scrollbar(mut self, state: Behaviour) -> Self {
        self.primary_scrollbar = state;
        self
    }

    /// enable or disable secondary scrollbar of the [`WrapBox`]
    pub fn secondary_scrollbar(mut self, state: Behaviour) -> Self {
        self.secondary_scrollbar = state;
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

impl<'a, Message, Renderer: text::Renderer> Default
    for WrapBox<'a, Message, Renderer>
where
    Renderer::Theme: WrapBoxStyleSheet,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message: 'a, Renderer> Widget<Message, Renderer>
    for WrapBox<'a, Message, Renderer>
where
    Renderer: text::Renderer + 'a,
    Renderer::Theme: WrapBoxStyleSheet,
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

    fn layout(&self, renderer: &Renderer, limits: &Limits) -> Node {
        let limits = limits
            .max_width(self.max_width)
            .max_height(self.max_height)
            .width(self.width)
            .height(self.height);

        let size = limits.fill();

        // skip the layout if it cannot be calculated efficiently
        // the nearest event will allow and trigger efficient layout
        if let Some(state) = self.state {
            self.create_layout(renderer, limits.fill(), &state)
        } else {
            Node::new(size)
        }
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: iced_native::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        let state = tree.state.downcast_ref();

        let mut owner = Node::default();
        let child =
            self.get_layout(renderer, layout, state, &mut owner, || {});

        let view_size = self.pad_size(layout.bounds().size());

        operation.container(None, &mut |operation| {
            self.visible(view_size, state)
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
        let state = tree.state.downcast_mut::<State>();
        if matches!(event, iced_native::Event::Mouse(_))
            && !layout.bounds().contains(cursor_position)
            && matches!(state.pressed, ScrollbarInteraction::None)
        {
            return iced_native::event::Status::Ignored;
        }

        let mut owner = Node::default();
        let child =
            self.get_layout(renderer, layout, state, &mut owner, || {
                shell.invalidate_layout();
            });

        let view_size = self.pad_size(layout.bounds().size());
        let view_bounds = Rectangle {
            width: view_size.width,
            height: view_size.height,
            ..child.bounds()
        };

        let content_size = child.bounds().size();
        let (first_o, last_o) = self.visible_pos(view_size, state);

        // scrolling
        if let iced_native::Event::Mouse(mouse::Event::WheelScrolled {
            delta,
        }) = event
        {
            match delta {
                ScrollDelta::Lines { x: _, y } => {
                    state.offset_y -= y * self.line_size();
                }
                ScrollDelta::Pixels { x: _, y } => {
                    state.offset_y -= y;
                }
            }
        }

        let mut captured = false;

        // mouse down
        if matches!(
            event,
            iced_native::Event::Mouse(mouse::Event::ButtonPressed(
                Button::Left
            ))
        ) {
            let bounds = layout.bounds();
            let (_, offset) = state.get_relative(view_size, content_size);
            let thumb_size = self.thumb_size(bounds, content_size);

            captured = true;

            if self.top_button_bounds(bounds).contains(cursor_position) {
                state.pressed = ScrollbarInteraction::Up;
            } else if self.bot_button_bounds(bounds).contains(cursor_position)
            {
                state.pressed = ScrollbarInteraction::Down;
            } else if self
                .thumb_bounds(bounds, thumb_size, offset)
                .contains(cursor_position)
            {
                state.pressed = ScrollbarInteraction::Thumb {
                    relative: offset,
                    cursor: cursor_position,
                };
            } else {
                let trough =
                    self.top_trough_bounds(bounds, thumb_size, offset);
                if trough.contains(cursor_position) {
                    let relative = offset * (cursor_position.y - trough.y)
                        / trough.height;
                    state.pressed = ScrollbarInteraction::Thumb {
                        relative,
                        cursor: cursor_position,
                    };
                    state.scroll_relative_y(view_size, content_size, relative);
                } else {
                    let trough =
                        self.bot_trough_bounds(bounds, thumb_size, offset);
                    if trough.contains(cursor_position) {
                        let relative = offset
                            + (1. - offset) * (cursor_position.y - trough.y)
                                / trough.height;
                        state.pressed = ScrollbarInteraction::Thumb {
                            relative,
                            cursor: cursor_position,
                        };
                        state.scroll_relative_y(
                            view_size,
                            content_size,
                            relative,
                        );
                    } else {
                        captured = false;
                    }
                }
            }
        }

        // mouse up
        if matches!(
            event,
            iced_native::Event::Mouse(mouse::Event::ButtonReleased(
                Button::Left
            ))
        ) {
            captured = true;

            match state.pressed {
                ScrollbarInteraction::Up => {
                    if self
                        .top_button_bounds(layout.bounds())
                        .contains(cursor_position)
                    {
                        state.offset_y -= view_bounds.height;
                    }
                }
                ScrollbarInteraction::Down => {
                    if self
                        .bot_button_bounds(layout.bounds())
                        .contains(cursor_position)
                    {
                        state.offset_y += view_bounds.height;
                    }
                }
                _ => captured = false,
            }

            state.pressed = ScrollbarInteraction::None;
        }

        // dragging
        if matches!(
            event,
            event::Event::Mouse(mouse::Event::CursorMoved { .. })
        ) {
            if let ScrollbarInteraction::Thumb { relative, cursor } =
                state.pressed
            {
                let bounds = layout.bounds();
                let relative = relative
                    + (cursor_position.y - cursor.y)
                        / (bounds.height
                            - self.scrollbar_button_height * 2.
                            - self.thumb_size(bounds, content_size));
                state.scroll_relative_y(view_size, content_size, relative);
                state.pressed = ScrollbarInteraction::Thumb {
                    relative,
                    cursor: cursor_position,
                };

                captured = true;
            }
        }

        state.offset_y = state
            .offset_y
            .min(content_size.height - view_size.height)
            .max(0.);

        let (first, last) = self.visible_pos(view_size, state);
        if first_o != first || last_o != last {
            shell.invalidate_layout();
        }

        self.state = Some(*state);

        if captured {
            return event::Status::Captured;
        }

        if matches!(event, iced_native::Event::Mouse(_))
            && !view_bounds.contains(cursor_position)
        {
            return iced_native::event::Status::Ignored;
        }

        self.visible_mut(view_size, state)
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
            return mouse::Interaction::Idle;
        }

        let state = tree.state.downcast_ref();

        let mut owner = Node::default();
        let child =
            self.get_layout(renderer, layout, state, &mut owner, || {});

        let view_size = self.pad_size(layout.bounds().size());
        let view_bounds = Rectangle {
            width: view_size.width,
            height: view_size.height,
            ..child.bounds()
        };

        if !view_bounds.contains(cursor_position) {
            return mouse::Interaction::Idle;
        }

        self.visible(view_size, state)
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
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        let mut owner = Node::default();
        let child =
            self.get_layout(renderer, layout, state, &mut owner, || {});

        let view_size = self.pad_size(layout.bounds().size());
        let view_bounds = Rectangle {
            width: view_size.width,
            height: view_size.height,
            ..child.bounds()
        };

        let (first, _) = self.visible_pos(view_size, state);

        let offset = if self.can_optimize() {
            let item_space = self.item_height + self.spacing_y;
            Vector::new(0., first as f32 * item_space - state.offset_y)
        } else {
            Vector::new(-state.offset_x, -state.offset_y)
        };

        let mouse_pos = if view_bounds.contains(cursor_position) {
            cursor_position - offset
        } else {
            // don't count with the mouse if it is outside
            Point::new(f32::INFINITY, f32::INFINITY)
        };
        let child_viewport = Rectangle {
            x: view_bounds.x - state.offset_x,
            y: view_bounds.y - state.offset_y,
            ..view_bounds
        };

        renderer.with_layer(view_bounds, |renderer| {
            renderer.with_translation(offset, |renderer| {
                for ((child, i), layout) in
                    self.visible(view_size, state).zip(child.children())
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

        if self.primary_scrollbar == Behaviour::Enabled {
            self.draw_scrollbar(
                layout,
                state,
                cursor_position,
                renderer,
                theme,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<iced_native::overlay::Element<'b, Message, Renderer>> {
        let state = tree.state.downcast_ref::<State>();

        let mut owner = Node::default();
        let child =
            self.get_layout(renderer, layout, state, &mut owner, || {});

        let view_size = self.pad_size(layout.bounds().size());
        let (first, last) = self.visible_pos(view_size, state);

        let children = self
            .visible_mut(view_size, state)
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

impl<'a, Message: 'a, Renderer: text::Renderer + 'a>
    From<WrapBox<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Renderer::Theme: WrapBoxStyleSheet,
{
    fn from(value: WrapBox<'a, Message, Renderer>) -> Self {
        Self::new(value)
    }
}

impl<'a, Message: 'a, Renderer: text::Renderer + 'a>
    WrapBox<'a, Message, Renderer>
where
    Renderer::Theme: WrapBoxStyleSheet,
{
    fn create_layout(
        &self,
        renderer: &Renderer,
        size: Size,
        state: &State,
    ) -> Node {
        let node = self.layout_wrap(renderer, size, state);
        Node::with_children(size, vec![node])
    }

    fn get_layout<'b: 'c, 'c, F: FnOnce()>(
        &self,
        renderer: &Renderer,
        layout: Layout<'b>,
        state: &State,
        owner: &'c mut Node,
        fun: F,
    ) -> Layout<'c> {
        match layout.children().next() {
            Some(c) => c,
            None => {
                // when the layout is not available, calculate temporary layout
                // so that there is no dropped event
                fun();
                *owner = self.create_layout(
                    renderer,
                    layout.bounds().size(),
                    state,
                );
                // owner.children().iter().next() is always Some
                Layout::new(owner.children().iter().next().unwrap())
            }
        }
    }

    #[inline]
    fn visible_pos(&self, view_size: Size, state: &State) -> (usize, usize) {
        if !self.can_optimize() {
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
    fn visible(
        &'a self,
        view_size: Size,
        state: &State,
    ) -> impl Iterator<Item = (&Element<'a, Message, Renderer>, usize)> {
        let (first, last) = self.visible_pos(view_size, state);
        self.children[first..=last]
            .iter()
            .enumerate()
            .map(move |(i, c)| (c, i + first))
    }

    #[inline]
    fn visible_mut(
        &mut self,
        view_size: Size,
        state: &State,
    ) -> impl Iterator<Item = (&mut Element<'a, Message, Renderer>, usize)>
    {
        let (first, last) = self.visible_pos(view_size, state);
        self.children[first..=last]
            .iter_mut()
            .enumerate()
            .map(move |(i, c)| (c, i + first))
    }

    fn layout_wrap(
        &self,
        renderer: &Renderer,
        size: Size,
        state: &State,
    ) -> Node {
        if self.can_optimize() {
            self.layout_wrap_optimized(renderer, size, state)
        } else {
            self.layout_wrap_general(renderer, size)
        }
    }

    fn layout_wrap_optimized(
        &self,
        renderer: &Renderer,
        size: Size,
        state: &State,
    ) -> Node {
        let size = self.pad_size(size);
        let item_lim =
            Limits::new(Size::ZERO, Size::new(size.width, self.item_height));
        let item_space_y = self.item_height + self.spacing_y;

        let children = self
            .visible(size, state)
            .enumerate()
            .map(|(i, (c, _))| {
                c.as_widget()
                    .layout(renderer, &item_lim)
                    .translate(Vector::new(0., item_space_y * i as f32))
            })
            .collect::<Vec<_>>();

        Node::with_children(
            Size::new(
                size.width,
                item_space_y * self.children.len() as f32 - self.spacing_y,
            ),
            children,
        )
        .translate(Vector::new(self.padding.left, self.padding.top))
    }

    fn layout_wrap_general(&self, renderer: &Renderer, size: Size) -> Node {
        let size = self.pad_size(size);
        let item_lim =
            Limits::new(Size::ZERO, Size::new(size.width, f32::MAX));

        let mut pos = 0.;
        let children = self
            .children
            .iter()
            .map(|c| {
                let node = c
                    .as_widget()
                    .layout(renderer, &item_lim)
                    .translate(Vector::new(0., pos));
                pos += node.size().height + self.spacing_y;
                node
            })
            .collect();

        Node::with_children(
            Size::new(size.width, pos - self.spacing_y),
            children,
        )
        .translate(Vector::new(self.padding.left, self.padding.top))
    }

    fn pad_size(&self, size: Size) -> Size {
        let mut size = Size::new(
            size.width - self.padding.left - self.padding.right,
            size.height - self.padding.top - self.padding.bottom,
        );

        if self.primary_scrollbar == Behaviour::Enabled {
            size.width -= self.scrollbar_width;
        }

        if self.secondary_scrollbar == Behaviour::Enabled {
            size.height -= self.scrollbar_width;
        }

        size
    }

    fn can_optimize(&self) -> bool {
        self.item_height > 0.
    }

    fn line_size(&self) -> f32 {
        if self.line_scroll == 0. {
            if self.item_height == 0. {
                DEFAULT_SCROLL_SPEED
            } else {
                self.item_height + self.spacing_y
            }
        } else {
            self.line_scroll
        }
    }

    fn top_button_bounds(&self, bounds: Rectangle) -> Rectangle {
        Rectangle {
            x: bounds.x + bounds.width - self.scrollbar_width,
            y: bounds.y,
            width: self.scrollbar_width,
            height: self.scrollbar_button_height,
        }
    }

    fn bot_button_bounds(&self, bounds: Rectangle) -> Rectangle {
        Rectangle {
            x: bounds.x + bounds.width - self.scrollbar_width,
            y: bounds.y + bounds.height - self.scrollbar_button_height,
            width: self.scrollbar_width,
            height: self.scrollbar_button_height,
        }
    }

    fn top_trough_bounds(
        &self,
        bounds: Rectangle,
        thumb_size: f32,
        offset: f32,
    ) -> Rectangle {
        Rectangle {
            x: bounds.x + bounds.width - self.scrollbar_width,
            y: bounds.y + self.scrollbar_button_height,
            width: self.scrollbar_width,
            height: (bounds.height
                - self.scrollbar_button_height * 2.
                - thumb_size)
                * offset,
        }
    }

    fn bot_trough_bounds(
        &self,
        bounds: Rectangle,
        thumb_size: f32,
        offset: f32,
    ) -> Rectangle {
        let y = bounds.y
            + (bounds.height - self.scrollbar_button_height * 2. - thumb_size)
                * offset
            + thumb_size
            + self.scrollbar_button_height;
        Rectangle {
            x: bounds.x + bounds.width - self.scrollbar_width,
            y,
            width: self.scrollbar_width,
            height: bounds.height - y - self.scrollbar_button_height,
        }
    }

    fn thumb_bounds(
        &self,
        bounds: Rectangle,
        thumb_size: f32,
        offset: f32,
    ) -> Rectangle {
        Rectangle {
            x: bounds.x + bounds.width - self.scrollbar_width,
            y: bounds.y
                + self.scrollbar_button_height
                + (bounds.height
                    - self.scrollbar_button_height * 2.
                    - thumb_size)
                    * offset,
            width: self.scrollbar_width,
            height: thumb_size,
        }
    }

    fn thumb_size(&self, bounds: Rectangle, content: Size) -> f32 {
        let trough_height = bounds.height - self.scrollbar_button_height * 2.;
        self.min_thumb_size
            .max(trough_height * bounds.height / content.height)
            .min(trough_height / 2.)
    }

    fn draw_scrollbar(
        &self,
        layout: Layout,
        state: &State,
        cursor_position: Point,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
    ) {
        let bounds = layout.bounds();
        // the childern.next will always be Some
        let view_size = self.pad_size(bounds.size());
        let content_bounds = layout.children().next().unwrap().bounds();
        let topleft =
            Point::new(bounds.width - self.scrollbar_width, bounds.y);
        let thumb_size = self.thumb_size(bounds, content_bounds.size());
        let (_, offset) = state.get_relative(view_size, content_bounds.size());

        let mo_wrap = bounds.contains(cursor_position);
        let mo_scroll = Rectangle::new(
            topleft,
            Size::new(self.scrollbar_width, bounds.height),
        )
        .contains(cursor_position);

        // draw the wrap_box background
        theme
            .background(
                &self.style,
                MousePos::from_bools(mo_wrap, mo_scroll, mo_wrap),
            )
            .draw(renderer, bounds);

        // draw the top scrollbar button
        let top_button = self.top_button_bounds(bounds);
        let pos = MousePos::from_bools(
            mo_wrap,
            mo_scroll,
            top_button.contains(cursor_position),
        );
        let b_style = theme.button_style(
            &self.style,
            pos,
            matches!(state.pressed, ScrollbarInteraction::Up),
            true,
            offset,
        );
        b_style.square.draw(renderer, top_button);
        // TODO: use better arrow than just the character '^'
        renderer.fill_text(text::Text {
            content: "^",
            bounds: Rectangle {
                x: top_button.x + 10.,
                y: top_button.y + 13.,
                ..top_button
            },
            size: self.scrollbar_button_height,
            color: b_style.foreground,
            font: Default::default(),
            horizontal_alignment: iced_native::alignment::Horizontal::Center,
            vertical_alignment: iced_native::alignment::Vertical::Center,
        });

        // draw the bottom scrollbar button
        let bottom_button = self.bot_button_bounds(bounds);
        let pos = MousePos::from_bools(
            mo_wrap,
            mo_scroll,
            bottom_button.contains(cursor_position),
        );
        let b_style = theme.button_style(
            &self.style,
            pos,
            matches!(state.pressed, ScrollbarInteraction::Down),
            false,
            offset,
        );
        b_style.square.draw(renderer, bottom_button);
        // TODO: use better arrow than just the character 'v'
        renderer.fill_text(text::Text {
            content: "v",
            bounds: Rectangle {
                x: bottom_button.x + 10.,
                y: bottom_button.y + 7.,
                ..bottom_button
            },
            size: self.scrollbar_button_height,
            color: b_style.foreground,
            font: Default::default(),
            horizontal_alignment: iced_native::alignment::Horizontal::Center,
            vertical_alignment: iced_native::alignment::Vertical::Center,
        });

        // draw the top trough
        if offset != 0. {
            let trough = self.top_trough_bounds(bounds, thumb_size, offset);
            let pos = MousePos::from_bools(
                mo_wrap,
                mo_scroll,
                trough.contains(cursor_position),
            );
            let quad = Quad {
                bounds: trough,
                border_radius: 0.0.into(),
                border_width: 0.,
                border_color: Color::BLACK,
            };
            renderer.fill_quad(
                quad,
                theme.trough_style(&self.style, pos, true, offset),
            );
        }

        // draw the bottom trough
        if offset != 1. {
            let trough = self.bot_trough_bounds(bounds, thumb_size, offset);
            let pos = MousePos::from_bools(
                mo_wrap,
                mo_scroll,
                trough.contains(cursor_position),
            );
            let quad = Quad {
                bounds: trough,
                border_radius: 0.0.into(),
                border_width: 0.,
                border_color: Color::BLACK,
            };
            renderer.fill_quad(
                quad,
                theme.trough_style(&self.style, pos, false, offset),
            );
        }

        // draw the thumb
        let thumb = self.thumb_bounds(bounds, thumb_size, offset);
        let pos = MousePos::from_bools(
            mo_wrap,
            mo_scroll,
            thumb.contains(cursor_position),
        );
        theme
            .thumb_style(
                &self.style,
                pos,
                matches!(
                    state.pressed,
                    ScrollbarInteraction::Thumb {
                        relative: _,
                        cursor: _
                    }
                ),
                offset,
            )
            .draw(renderer, thumb);
    }
}

pub fn wrap_box<Message, Renderer: text::Renderer>(
    childern: Vec<Element<'_, Message, Renderer>>,
) -> WrapBox<'_, Message, Renderer>
where
    Renderer::Theme: WrapBoxStyleSheet,
{
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
    pressed: ScrollbarInteraction,
}

impl State {
    fn new() -> Self {
        Self {
            offset_x: 0.,
            offset_y: 0.,
            pressed: ScrollbarInteraction::None,
        }
    }

    fn get_relative(&self, view_size: Size, content_size: Size) -> (f32, f32) {
        (
            self.offset_x / (content_size.width - view_size.width),
            self.offset_y / (content_size.height - view_size.height),
        )
    }

    fn scroll_relative_y(
        &mut self,
        view_size: Size,
        content_size: Size,
        pos: f32,
    ) {
        self.offset_y = (content_size.height - view_size.height) * pos;
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
enum ScrollbarInteraction {
    None,
    Up,
    Down,
    Thumb { relative: f32, cursor: Point },
}

pub enum Behaviour {
    Enabled,
    Disabled,
    Hidden,
}

impl PartialEq for Behaviour {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

pub trait WrapBoxStyleSheet {
    type Style: Default;

    fn background(&self, style: &Self::Style, pos: MousePos) -> SquareStyle;

    fn button_style(
        &self,
        style: &Self::Style,
        pos: MousePos,
        pressed: bool,
        is_start: bool,
        relative_scroll: f32,
    ) -> ButtonStyle;

    fn thumb_style(
        &self,
        style: &Self::Style,
        pos: MousePos,
        pressed: bool,
        relative_scroll: f32,
    ) -> SquareStyle;

    fn trough_style(
        &self,
        style: &Self::Style,
        pos: MousePos,
        is_start: bool,
        relative_scroll: f32,
    ) -> Background;
}

pub enum MousePos {
    DirectlyOver,
    OverScrollbar,
    OverWrapBox,
    None,
}

impl MousePos {
    fn from_bools(wrap: bool, scroll: bool, directly: bool) -> Self {
        if directly {
            Self::DirectlyOver
        } else if scroll {
            Self::OverScrollbar
        } else if wrap {
            Self::OverWrapBox
        } else {
            Self::None
        }
    }
}

impl PartialEq for MousePos {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

pub struct SquareStyle {
    background: Background,
    border: Color,
    border_thickness: f32,
    border_radius: BorderRadius,
}

impl SquareStyle {
    fn draw<Renderer: text::Renderer>(
        &self,
        renderer: &mut Renderer,
        bounds: Rectangle,
    ) {
        renderer.fill_quad(
            Quad {
                bounds,
                border_radius: self.border_radius,
                border_width: self.border_thickness,
                border_color: self.border,
            },
            self.background,
        )
    }
}

pub struct ButtonStyle {
    square: SquareStyle,
    foreground: Color,
}

impl WrapBoxStyleSheet for Theme {
    type Style = ();

    fn background(&self, _style: &Self::Style, _pos: MousePos) -> SquareStyle {
        SquareStyle {
            background: Background::Color(Color::TRANSPARENT),
            border: Color::BLACK,
            border_thickness: 0.,
            border_radius: 0.0.into(),
        }
    }

    fn button_style(
        &self,
        style: &Self::Style,
        pos: MousePos,
        pressed: bool,
        is_start: bool,
        relative_scroll: f32,
    ) -> ButtonStyle {
        let square = self.thumb_style(style, pos, pressed, relative_scroll);

        if is_start && relative_scroll == 0.
            || !is_start && relative_scroll == 1.
        {
            // inactive
            ButtonStyle {
                square,
                foreground: color!(0x777777),
            }
        } else {
            // active
            ButtonStyle {
                square,
                foreground: color!(0xEEEEEE),
            }
        }
    }

    fn thumb_style(
        &self,
        _style: &Self::Style,
        pos: MousePos,
        pressed: bool,
        _relative_scroll: f32,
    ) -> SquareStyle {
        let square = SquareStyle {
            background: Background::Color(color!(0x333333)),
            border: color!(0x555555),
            border_thickness: 0.,
            border_radius: 0.0.into(),
        };

        if pressed {
            SquareStyle {
                background: Background::Color(color!(0x555555)),
                ..square
            }
        } else if pos == MousePos::DirectlyOver {
            SquareStyle {
                background: Background::Color(color!(0x444444)),
                ..square
            }
        } else {
            square
        }
    }

    fn trough_style(
        &self,
        _style: &Self::Style,
        _pos: MousePos,
        _is_start: bool,
        _relative_scroll: f32,
    ) -> Background {
        Background::Color(color!(0x222222))
    }
}
