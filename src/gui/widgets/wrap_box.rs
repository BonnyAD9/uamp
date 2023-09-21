use std::{cell::Cell, vec};

use iced_core::{
    color, event,
    layout::{Limits, Node},
    mouse::{self, Button, ScrollDelta},
    overlay::Group,
    renderer::{self, Quad},
    svg,
    widget::{self, tree, Tree},
    Background, BorderRadius, Clipboard, Color, Element, Event, Layout,
    Length, Padding, Pixels, Point, Rectangle, Shell, Size, Vector, Widget,
};

use iced::Theme;

use self::ItemDirection::{
    BottomToTop, LeftToRight, RightToLeft, TopToBottom,
};

use super::icons;

pub const DEFAULT_SCROLL_SPEED: f32 = 60.;
pub const DEFAULT_SCROLLBAR_WIDTH: f32 = 20.;
pub const DEFAULT_SCROLLBAR_BUTTON_HEIGHT: f32 = 20.;
pub const DEFAULT_MIN_THUMB_SIZE: f32 = 20.;

/// Container that distributes its contents both vertically and horizontaly
/// and also has a scollbar. Advantage over normal scrollbar combined with
/// row or column is that this more efficiently handles large amounts of
/// childern. In the first versions it may not support the horizontal part.
///
/// This is not finished and currently supports only TopToBottom vertical
/// scrolling
pub struct WrapBox<'a, Message, Renderer: svg::Renderer>
where
    Renderer::Theme: StyleSheet,
{
    /// amount of space between items on the x axis
    spacing_x: f32,
    /// amount of space between items on the y axis
    spacing_y: f32,
    /// padding of the viewport of the items
    padding: Padding,
    /// width of the whole [`WrapBox`], [`Length::Shrink`] may not work properly
    width: Length,
    /// height of the [`WrapBox`] [`Length::Shrink`] may not work properly
    height: Length,
    /// max width of the [`WrapBox`]
    max_width: f32,
    /// max height of the [`WrapBox`]
    max_height: f32,
    /// fixed width of items, 0 means that the width is not fixed
    item_width: f32,
    /// fixed height of items, 0 means that the height is not fixed
    item_height: f32,
    /// max number of items in line, then it will wrap
    max_wrap: u32,
    /// min number of items on one line, it will not wrap before
    min_wrap: u32,
    /// number of pixels per one line scroll
    line_scroll: f32,
    /// width of the scrollbar
    scrollbar_width: f32,
    /// height of the buttons on the scrollbar
    scrollbar_button_height: f32,
    /// min size of the scrollbar
    min_thumb_size: f32,
    /// primary direction of the items
    primary_direction: ItemDirection,
    /// secondary direction of the items, lines to wrap
    secondary_direction: ItemDirection,
    /// when the primary scrollbar should be shown
    primary_scrollbar: Behaviour,
    /// when the secondary scrollbar should be shown
    secondary_scrollbar: Behaviour,
    /// the items
    children: Vec<Element<'a, Message, Renderer>>,
    /// the state of the scrollbar, set only when up to date
    state: &'a Cell<State>,
    /// style of the [`WrapBox`]
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer: svg::Renderer> WrapBox<'a, Message, Renderer>
where
    Renderer::Theme: StyleSheet,
{
    /// creates empty [`WrapBox`]
    pub fn new(state: &'a Cell<State>) -> Self {
        Self::with_children(Vec::new(), state)
    }

    /// creates a [`WrapBox`] with the given elements
    pub fn with_children(
        childern: Vec<Element<'a, Message, Renderer>>,
        state: &'a Cell<State>,
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
            primary_scrollbar: Behaviour::Hidden,
            secondary_scrollbar: Behaviour::Disabled,
            children: childern,
            state,
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

    /// sets the width of the scrollbar (this is the height of the horizontal
    /// scrollbar), default value is [`DEFAULT_SCROLLBAR_WIDTH`]
    pub fn scrollbar_width(mut self, width: impl Into<Pixels>) -> Self {
        self.scrollbar_width = width.into().0;
        self
    }

    /// sets the height of the scrollbar buttons (this is width of the buttons
    /// on the horizontal scrollbar), default value is
    /// [`DEFAULT_SCROLLBAR_BUTTON_HEIGHT`]
    pub fn scrollbar_button_height(
        mut self,
        height: impl Into<Pixels>,
    ) -> Self {
        self.scrollbar_button_height = height.into().0;
        self
    }

    /// sets the width of the scrollbar (this is the height of the horizontal
    /// scrollbar), default valueis [`DEFAULT_MIN_THUMB_SIZE`]
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

    pub fn style(
        mut self,
        style: <Renderer::Theme as StyleSheet>::Style,
    ) -> Self {
        self.style = style;
        self
    }

    pub fn from_layout_style(
        mut self,
        style: &impl LayoutStyleSheet<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        let style = style.layout(&self.style);
        style.spacing.0.set_if(&mut self.spacing_x);
        style.spacing.1.set_if(&mut self.spacing_y);
        style.padding.set_if(&mut self.padding);
        style.item_size.0.set_if(&mut self.item_width);
        style.item_size.1.set_if(&mut self.item_height);
        style.scrollbar_width.set_if(&mut self.scrollbar_width);
        style
            .scrollbar_button_height
            .set_if(&mut self.scrollbar_button_height);
        style.min_thumb_size.set_if(&mut self.min_thumb_size);
        style.primary_direction.set_if(&mut self.primary_direction);
        style
            .secondary_direction
            .set_if(&mut self.secondary_direction);
        style.primary_scrollbar.set_if(&mut self.primary_scrollbar);
        style
            .secondary_scrollbar
            .set_if(&mut self.secondary_scrollbar);
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

/*impl<'a, Message, Renderer: svg::Renderer> Default
    for WrapBox<'a, Message, Renderer>
where
    Renderer::Theme: StyleSheet,
{
    fn default() -> Self {
        Self::new()
    }
}*/

impl<'a, Message: 'a, Renderer> Widget<Message, Renderer>
    for WrapBox<'a, Message, Renderer>
where
    Renderer: svg::Renderer + 'a,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn width(&self) -> iced_core::Length {
        self.width
    }

    fn height(&self) -> iced_core::Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &Limits) -> Node {
        let limits = limits
            .max_width(self.max_width)
            .max_height(self.max_height)
            .width(self.width)
            .height(self.height);
        let size = limits.fill();
        let view_size = self.pad_size(size);

        let mut state = self.state.get();
        if let Some(i) = state.scroll_to {
            if !self.is_item_visible(view_size, i) {
                state.offset_y = self.item_scroll_pos(view_size, i);
                self.state.set(state);
            }
        }

        self.create_layout(renderer, size, &state)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: iced_core::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        let state = self.state.get();

        let view_size = self.pad_size(layout.bounds().size());
        let child = layout.children().next().unwrap();

        operation.container(None, layout.bounds(), &mut |operation| {
            self.visible(view_size, &state)
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
        event: Event,
        layout: Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let mut state = self.state.get();

        let child = layout.children().next().unwrap();

        let view_size = self.pad_size(layout.bounds().size());
        let view_bounds = Rectangle {
            width: view_size.width,
            height: view_size.height,
            ..child.bounds()
        };

        let content_size = child.bounds().size();
        let (start_o, end_o) = self.visible_range(view_size, &state);

        // scrolling
        if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
            match delta {
                ScrollDelta::Lines { x: _, y } => {
                    state.offset_y -= y * self.line_size();
                }
                ScrollDelta::Pixels { x: _, y } => {
                    state.offset_y -= y;
                }
            }
            state.scroll_to = None;
        }

        let mut captured = false;

        // mouse down
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(Button::Left))
        ) {
            let bounds = layout.bounds();
            let (_, offset) = state.get_relative(view_size, content_size);
            let thumb_size = self.thumb_size(bounds, content_size);

            captured = true;

            if cursor.is_over(self.top_button_bounds(bounds)) {
                state.pressed = ScrollbarInteraction::Up;
            } else if cursor.is_over(self.bot_button_bounds(bounds)) {
                state.pressed = ScrollbarInteraction::Down;
            } else if cursor
                .is_over(self.thumb_bounds(bounds, thumb_size, offset))
            {
                if let Some(cursor) = cursor.position() {
                    state.pressed = ScrollbarInteraction::Thumb {
                        relative: offset,
                        cursor,
                    };
                }
            } else {
                let trough =
                    self.top_trough_bounds(bounds, thumb_size, offset);
                if cursor.is_over(trough) {
                    if let Some(cursor_position) = cursor.position() {
                        let relative = offset * (cursor_position.y - trough.y)
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
                    }
                } else {
                    let trough =
                        self.bot_trough_bounds(bounds, thumb_size, offset);
                    if cursor.is_over(trough) {
                        if let Some(cursor_position) = cursor.position() {
                            let relative = offset
                                + (1. - offset)
                                    * (cursor_position.y - trough.y)
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
                        }
                    } else {
                        captured = false;
                    }
                }
            }
        }

        // mouse up
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonReleased(Button::Left))
        ) {
            captured = true;

            match state.pressed {
                ScrollbarInteraction::Up => {
                    if cursor.is_over(self.top_button_bounds(layout.bounds()))
                    {
                        state.offset_y -= view_bounds.height;
                    }
                }
                ScrollbarInteraction::Down => {
                    if cursor.is_over(self.bot_button_bounds(layout.bounds()))
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
            if let (
                ScrollbarInteraction::Thumb { relative, cursor },
                Some(cursor_position),
            ) = (state.pressed, cursor.position())
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

        if state.pressed != ScrollbarInteraction::None {
            state.scroll_to = None;
        }

        state.offset_y = state
            .offset_y
            .min(content_size.height - view_size.height)
            .max(0.);

        let (start, end) = self.visible_range(view_size, &state);
        if start_o != start || end_o != end {
            shell.invalidate_layout();
        }

        self.state.set(state);

        if captured {
            return event::Status::Captured;
        }

        if matches!(event, Event::Mouse(_)) && !cursor.is_over(view_bounds) {
            return event::Status::Ignored;
        }

        let (cor_x, cor_y) = self.scroll_offset(&state);

        let cursor_position = cursor
            .position()
            .map(|cursor_position| {
                mouse::Cursor::Available(Point::new(
                    cursor_position.x + cor_x,
                    cursor_position.y + cor_y,
                ))
            })
            .unwrap_or(mouse::Cursor::Unavailable);

        self.visible_mut(view_size, &state)
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
                    &layout.bounds(),
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let cursor_position =
            if let iced_core::mouse::Cursor::Available(cursor_position) =
                cursor
            {
                cursor_position
            } else {
                return mouse::Interaction::Idle;
            };

        if !layout.bounds().contains(cursor_position) {
            return mouse::Interaction::Idle;
        }

        let state = self.state.get();

        let child = layout.children().next().unwrap();

        let view_size = self.pad_size(layout.bounds().size());
        let view_bounds = Rectangle {
            width: view_size.width,
            height: view_size.height,
            ..child.bounds()
        };

        if !view_bounds.contains(cursor_position) {
            return mouse::Interaction::Idle;
        }

        let (cor_x, cor_y) = self.scroll_offset(&state);

        let cursor_position =
            Point::new(cursor_position.x + cor_x, cursor_position.y + cor_y);

        self.visible(view_size, &state)
            .zip(child.children())
            .map(|((child, i), layout)| {
                child.as_widget().mouse_interaction(
                    &tree.children[i],
                    layout,
                    mouse::Cursor::Available(cursor_position),
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
        theme: &<Renderer as iced_core::Renderer>::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = self.state.get();

        let child = layout.children().next().unwrap();

        let view_size = self.pad_size(layout.bounds().size());
        let view_bounds = Rectangle {
            width: view_size.width,
            height: view_size.height,
            ..child.bounds()
        };

        let (start, _) = self.visible_range(view_size, &state);

        let offset = if self.can_optimize() {
            let item_space = self.item_height + self.spacing_y;
            Vector::new(0., start as f32 * item_space - state.offset_y)
        } else {
            Vector::new(-state.offset_x, -state.offset_y)
        };

        let mouse_pos = match cursor {
            mouse::Cursor::Available(cursor_position) => {
                if view_bounds.contains(cursor_position) {
                    mouse::Cursor::Available(cursor_position - offset)
                } else {
                    // don't count with the mouse if it is outside
                    mouse::Cursor::Unavailable
                }
            }
            mouse::Cursor::Unavailable => mouse::Cursor::Unavailable,
        };

        let child_viewport = Rectangle {
            x: view_bounds.x - state.offset_x,
            y: view_bounds.y - state.offset_y,
            ..view_bounds
        };

        renderer.with_layer(view_bounds, |renderer| {
            renderer.with_translation(offset, |renderer| {
                for ((child, i), layout) in
                    self.visible(view_size, &state).zip(child.children())
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

        let draw_scroll = match self.primary_scrollbar {
            Behaviour::Enabled => true,
            Behaviour::Hidden => child.bounds().height > view_bounds.height,
            Behaviour::Disabled => false,
        };

        self.draw_scrollbar(
            child,
            layout.bounds(),
            &state,
            cursor,
            renderer,
            theme,
            draw_scroll,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<iced_core::overlay::Element<'b, Message, Renderer>> {
        let state = self.state.get();

        let child = layout.children().next().unwrap();

        let view_size = self.pad_size(layout.bounds().size());
        let (start, end) = self.visible_range(view_size, &state);

        let children = self
            .visible_mut(view_size, &state)
            .zip(&mut tree.children[start..end])
            .zip(child.children())
            .filter_map(|(((child, _), state), layout)| {
                child.as_widget_mut().overlay(state, layout, renderer)
            })
            .collect::<Vec<_>>();

        (!children.is_empty())
            .then(|| Group::with_children(children).overlay())
    }
}

impl<'a, Message: 'a, Renderer: svg::Renderer + 'a>
    From<WrapBox<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Renderer::Theme: StyleSheet,
{
    fn from(value: WrapBox<'a, Message, Renderer>) -> Self {
        Self::new(value)
    }
}

impl<'a, Message: 'a, Renderer: svg::Renderer + 'a>
    WrapBox<'a, Message, Renderer>
where
    Renderer::Theme: StyleSheet,
{
    fn is_item_visible(&self, view_size: Size, i: usize) -> bool {
        let size =
            Size::new(view_size.width, view_size.height - self.item_height);
        let (start, end) = self.visible_range(size, &self.state.get());
        (start..end).contains(&i)
    }

    fn item_scroll_pos(&self, view_size: Size, i: usize) -> f32 {
        let item_pos = (self.item_height + self.spacing_y) * i as f32;
        let center_offset = (view_size.height - self.item_height) / 2.;
        let ideal = item_pos - center_offset;
        let content_size = (self.item_height + self.spacing_y)
            * self.children.len() as f32
            - self.spacing_y;
        ideal.max(0.).min(content_size - view_size.height)
    }

    /// creates the [`WrapBox`] layout, immidiate node is the bounds of the
    /// whole [`WrapBox`], it contains node with bounds of the viewport, it
    /// than contains the childern
    fn create_layout(
        &self,
        renderer: &Renderer,
        size: Size,
        state: &State,
    ) -> Node {
        let node = self.layout_wrap(renderer, size, state);
        Node::with_children(size, vec![node])
    }

    /// gets the range of the visible childern indexes (inclusive - exclusive)
    #[inline]
    fn visible_range(&self, view_size: Size, state: &State) -> (usize, usize) {
        if !self.can_optimize() {
            (0, self.children.len())
        } else {
            let item_space = self.item_height + self.spacing_y;
            (
                (state.offset_y / item_space).max(0.) as usize,
                (((state.offset_y + view_size.height) / item_space).ceil()
                    as usize)
                    .min(self.children.len()),
            )
        }
    }

    /// gets iterator over the visible childern
    #[inline]
    fn visible(
        &'a self,
        view_size: Size,
        state: &State,
    ) -> impl Iterator<Item = (&Element<'a, Message, Renderer>, usize)> {
        let (start, end) = self.visible_range(view_size, state);
        self.children[start..end]
            .iter()
            .enumerate()
            .map(move |(i, c)| (c, i + start))
    }

    /// gets mutable iterator over the visible children
    #[inline]
    fn visible_mut(
        &mut self,
        view_size: Size,
        state: &State,
    ) -> impl Iterator<Item = (&mut Element<'a, Message, Renderer>, usize)>
    {
        let (start, end) = self.visible_range(view_size, state);
        self.children[start..end]
            .iter_mut()
            .enumerate()
            .map(move |(i, c)| (c, i + start))
    }

    /// creates the [`WrapBox`] layout, immidiate node is the
    /// bounds of the viewport, it
    /// than contains the childern
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

    /// creates the [`WrapBox`] layout in a optimized way, immidiate node is
    /// the bounds of the viewport, it than contains the childern. Works only
    /// if `can_optimize()` returns true.
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

    /// creates the [`WrapBox`] layout in a general way, immidiate node is
    /// the bounds of the viewport, it than contains the childern. It may be
    /// very slow, use `layout_wrap_optimized` when possible.
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

    /// Returns the size of object with the the padding
    fn pad_size(&self, size: Size) -> Size {
        let mut size = Size::new(
            size.width - self.padding.left - self.padding.right,
            size.height - self.padding.top - self.padding.bottom,
        );

        if self.primary_scrollbar != Behaviour::Disabled {
            size.width -= self.scrollbar_width;
        }

        if self.secondary_scrollbar != Behaviour::Disabled {
            size.height -= self.scrollbar_width;
        }

        size
    }

    /// Returns true when actions such as layouting can be optimized
    fn can_optimize(&self) -> bool {
        self.item_height > 0.
    }

    /// gets the size of line when scrolling
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

    /// Calculates the bounds of the top scrollbar buttion from the [`WrapBox`]
    /// bounds.
    fn top_button_bounds(&self, bounds: Rectangle) -> Rectangle {
        Rectangle {
            x: bounds.x + bounds.width - self.scrollbar_width,
            y: bounds.y,
            width: self.scrollbar_width,
            height: self.scrollbar_button_height,
        }
    }

    /// Calculates the bounds of the bottom scrollbar buttion from the
    /// [`WrapBox`] bounds.
    fn bot_button_bounds(&self, bounds: Rectangle) -> Rectangle {
        Rectangle {
            x: bounds.x + bounds.width - self.scrollbar_width,
            y: bounds.y + bounds.height - self.scrollbar_button_height,
            width: self.scrollbar_width,
            height: self.scrollbar_button_height,
        }
    }

    /// Calculates the bounds of the top trough from the [`WrapBox`] bounds and
    /// thumb offset and size
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

    /// Calculates the bounds of the bottom trough from the [`WrapBox`] bounds
    /// and thumb offset and size
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
            height: bounds.height - y - self.scrollbar_button_height
                + bounds.y,
        }
    }

    /// Calculates the thumb bounds from the [`WrapBox`] bounds, thumb size and
    /// its offset
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

    /// Callculates the thumb size from the [`WrapBox`] bounds and size of the
    /// contents
    fn thumb_size(&self, bounds: Rectangle, content: Size) -> f32 {
        let trough_height = bounds.height - self.scrollbar_button_height * 2.;
        self.min_thumb_size
            .max(trough_height * bounds.height / content.height)
            .min(trough_height)
    }

    /// Calculates the scroll offset of the contents in pixels
    fn scroll_offset(&self, state: &State) -> (f32, f32) {
        (
            if self.item_width == 0. {
                state.offset_x
            } else {
                state.offset_x % (self.item_width + self.spacing_x)
            },
            if self.item_height == 0. {
                state.offset_y
            } else {
                state.offset_y % (self.item_height + self.spacing_y)
            },
        )
    }

    /// Draws the scrollbar of a [`WrapBox`]
    fn draw_scrollbar(
        &self,
        layout: Layout,
        bounds: Rectangle,
        state: &State,
        cursor: mouse::Cursor,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        draw_scroll: bool,
    ) {
        // the childern.next will always be Some
        let view_size = self.pad_size(bounds.size());
        let content_bounds = layout.bounds();
        let topleft =
            Point::new(bounds.width - self.scrollbar_width, bounds.y);
        let thumb_size = self.thumb_size(bounds, content_bounds.size());
        let (_, offset) = state.get_relative(view_size, content_bounds.size());

        let mo_wrap = cursor.is_over(bounds);
        let mo_scroll = cursor.is_over(Rectangle::new(
            topleft,
            Size::new(self.scrollbar_width, bounds.height),
        ));

        // draw the wrap_box background
        theme
            .background(
                &self.style,
                MousePos::from_bools(mo_wrap, mo_scroll, mo_wrap),
            )
            .draw(renderer, bounds);

        if !draw_scroll {
            return;
        }

        // draw the top scrollbar button
        let top_button = self.top_button_bounds(bounds);
        let pos = MousePos::from_bools(
            mo_wrap,
            mo_scroll,
            cursor.is_over(top_button),
        );
        let b_style = theme.button_style(
            &self.style,
            pos,
            matches!(state.pressed, ScrollbarInteraction::Up),
            true,
            offset,
        );
        b_style.square.draw(renderer, top_button);
        renderer.draw(
            icons::POINT_UP.into(),
            Some(b_style.foreground),
            top_button,
        );

        // draw the bottom scrollbar button
        let bottom_button = self.bot_button_bounds(bounds);
        let pos = MousePos::from_bools(
            mo_wrap,
            mo_scroll,
            cursor.is_over(bottom_button),
        );
        let b_style = theme.button_style(
            &self.style,
            pos,
            matches!(state.pressed, ScrollbarInteraction::Down),
            false,
            offset,
        );
        b_style.square.draw(renderer, bottom_button);
        renderer.draw(
            icons::POINT_DOWN.into(),
            Some(b_style.foreground),
            bottom_button,
        );

        // draw the top trough
        if offset != 0. {
            let trough = self.top_trough_bounds(bounds, thumb_size, offset);
            let pos = MousePos::from_bools(
                mo_wrap,
                mo_scroll,
                cursor.is_over(trough),
            );
            let trough = Rectangle {
                height: trough.height + thumb_size / 2.,
                ..trough
            };
            theme
                .trough_style(&self.style, pos, true, offset)
                .draw(renderer, trough);
        }

        // draw the bottom trough
        if offset != 1. {
            let trough = self.bot_trough_bounds(bounds, thumb_size, offset);
            let pos = MousePos::from_bools(
                mo_wrap,
                mo_scroll,
                cursor.is_over(trough),
            );
            let trough = Rectangle {
                y: trough.y - thumb_size / 2.,
                height: trough.height + thumb_size / 2.,
                ..trough
            };
            theme
                .trough_style(&self.style, pos, false, offset)
                .draw(renderer, trough);
        }

        // draw the thumb
        let thumb = self.thumb_bounds(bounds, thumb_size, offset);
        let pos =
            MousePos::from_bools(mo_wrap, mo_scroll, cursor.is_over(thumb));
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

/// Determines the direction of layouting the items in [`WrapBox`]
#[derive(Copy, Clone)]
pub enum ItemDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

/// Contains the state of a [`WrapBox`]
#[derive(Copy, Clone)]
pub struct State {
    /// Absolute offset on the x axis
    offset_x: f32,
    /// Absolute offset on the y axis
    offset_y: f32,
    pressed: ScrollbarInteraction,
    /// Index of item to which the scrollbar should scroll, not supported in
    /// unoptimized mode
    pub scroll_to: Option<usize>,
}

impl State {
    pub fn scroll_to_top(&mut self) {
        self.offset_y = 0.;
    }

    /// Creates new [`WrapBox`] state, (not scrolled and not pressed)
    fn new() -> Self {
        Self {
            offset_x: 0.,
            offset_y: 0.,
            pressed: ScrollbarInteraction::None,
            scroll_to: None,
        }
    }

    /// Gets the relative offset of the scrollbars
    fn get_relative(&self, view_size: Size, content_size: Size) -> (f32, f32) {
        (
            self.offset_x / (content_size.width - view_size.width),
            self.offset_y / (content_size.height - view_size.height),
        )
    }

    /// scrolls to the given relative position of the scrollbar
    ///
    /// Pos should be value in range `0..=1`
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
    /// Creates the default [`WrapBox`] state (not scrolled and not pressed)
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ScrollbarInteraction {
    // nothing pressed
    None,
    // The top button is pressed
    Up,
    // The bottom button is pressed
    Down,
    // The thumb is pressed
    Thumb {
        /// Last relative offset of the scrollbar
        relative: f32,
        /// Last position of the mouse
        cursor: Point,
    },
}

/// Defines the behaviour of the scrollbar (its visibility)
#[derive(Clone, Copy)]
pub enum Behaviour {
    /// Scrollbar is always visible
    Enabled,
    /// Scrollbar is never visible
    Disabled,
    /// Scrollbar is visible only when necesary
    Hidden,
}

impl PartialEq for Behaviour {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

/// Defines the style of a [`WrapBox`]
pub trait StyleSheet {
    /// Type identifying the style
    type Style: Default;

    /// Determines how should be the background drawn
    fn background(&self, style: &Self::Style, pos: MousePos) -> SquareStyle;

    /// Defines how should the scrollbar buttons be drawn
    fn button_style(
        &self,
        style: &Self::Style,
        pos: MousePos,
        pressed: bool,
        is_start: bool,
        relative_scroll: f32,
    ) -> ButtonStyle;

    /// Defines the style of the thumb
    fn thumb_style(
        &self,
        style: &Self::Style,
        pos: MousePos,
        pressed: bool,
        relative_scroll: f32,
    ) -> SquareStyle;

    /// Defines the style of the troughs
    fn trough_style(
        &self,
        style: &Self::Style,
        pos: MousePos,
        is_start: bool,
        relative_scroll: f32,
    ) -> SquareStyle;
}

/// Shows mouse position relative to the scrollbar element
pub enum MousePos {
    /// Mouse is directly over the element
    DirectlyOver,
    /// Mouse is not directly over the element, but is somwhere over the
    /// scrollbar
    OverScrollbar,
    /// Mouse is not over the scrollbarm, but is somewhere over the [`WrapBox`]
    OverWrapBox,
    /// Mouse is not over the [`WrapBox`]
    None,
}

impl MousePos {
    /// Creates [`MousePos`] from the parts over which the mouse is
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

/// Defines how a square area should be drawn
pub struct SquareStyle {
    /// background of the area
    pub background: Background,
    /// Color of the border of the area
    pub border: Color,
    /// Thickness of the border around the area
    pub border_thickness: f32,
    /// Radious of the border corners
    pub border_radius: BorderRadius,
}

impl SquareStyle {
    /// Draws the square on the given position based on its style
    fn draw<Renderer: svg::Renderer>(
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

/// Style of a button
pub struct ButtonStyle {
    /// How the containing rectangle should be drawn
    pub square: SquareStyle,
    /// Color of the text of the button
    pub foreground: Color,
}

/// Default style of [`WrapBox`]
impl StyleSheet for Theme {
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
    ) -> SquareStyle {
        SquareStyle {
            background: Background::Color(color!(0x222222)),
            border: Color::BLACK,
            border_thickness: 0.,
            border_radius: 0.0.into(),
        }
    }
}

/// Layout Style for [`WrapBox`], None means don't change the value
pub struct LayoutStyle {
    /// horizontal and vertical spacing of the items
    pub spacing: (Option<f32>, Option<f32>),
    /// Padding of the items viewport
    pub padding: Option<Padding>,
    /// Size of the items
    pub item_size: (Option<f32>, Option<f32>),
    /// Width of the scrollbar
    pub scrollbar_width: Option<f32>,
    /// height of the scrollbar buttons
    pub scrollbar_button_height: Option<f32>,
    /// Minimal thumb size
    pub min_thumb_size: Option<f32>,
    /// primary direction of the items
    pub primary_direction: Option<ItemDirection>,
    /// secondary direction of the items
    pub secondary_direction: Option<ItemDirection>,
    /// Visibility of the primary scrollbar
    pub primary_scrollbar: Option<Behaviour>,
    /// Visibility of the secondary scrollbar
    pub secondary_scrollbar: Option<Behaviour>,
}

impl LayoutStyle {
    /// Default layout style
    pub fn empty() -> Self {
        LayoutStyle {
            spacing: (None, None),
            padding: None,
            item_size: (None, None),
            scrollbar_width: None,
            scrollbar_button_height: None,
            min_thumb_size: None,
            primary_direction: None,
            secondary_direction: None,
            primary_scrollbar: None,
            secondary_scrollbar: None,
        }
    }
}

impl Default for LayoutStyle {
    fn default() -> Self {
        LayoutStyle {
            scrollbar_width: Some(DEFAULT_SCROLLBAR_WIDTH),
            scrollbar_button_height: Some(DEFAULT_SCROLLBAR_BUTTON_HEIGHT),
            min_thumb_size: Some(DEFAULT_MIN_THUMB_SIZE),
            ..Self::empty()
        }
    }
}

/// Can create the [`LayoutStyle`] of [`WrapBox`]
pub trait LayoutStyleSheet<Style> {
    /// Creates the [`LayoutStyle`] of [`WrapBox`]
    fn layout(&self, style: &Style) -> LayoutStyle;
}

/// Default layout style implementation
impl<Style> LayoutStyleSheet<Style> for Theme {
    fn layout(&self, _style: &Style) -> LayoutStyle {
        LayoutStyle::default()
    }
}

/// Extension of [`Option<T>`]
trait OptionCopy<T>
where
    T: Copy,
{
    /// Sets field if the option is [`Some`]
    fn set_if(&self, field: &mut T);
}

impl<T> OptionCopy<T> for Option<T>
where
    T: Copy,
{
    fn set_if(&self, field: &mut T) {
        if let Some(v) = self {
            *field = *v;
        }
    }
}
