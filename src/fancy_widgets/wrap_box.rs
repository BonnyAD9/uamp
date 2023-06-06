use std::{backtrace, vec};

use iced_native::{
    event,
    layout::{Limits, Node},
    mouse::{self, ScrollDelta},
    overlay::Group,
    renderer::BorderRadius,
    widget::{self, tree, Tree},
    Background, Color, Element, Layout, Length, Padding, Pixels, Point,
    Rectangle, Size, Vector, Widget,
};

use self::ItemDirection::{
    BottomToTop, LeftToRight, RightToLeft, TopToBottom,
};

pub const DEFAULT_SCROLL_SPEED: f32 = 60.;
pub const DEFAULT_SCROLL_WIDTH: f32 = 20.;

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
    line_scroll: f32,
    primary_direction: ItemDirection,
    secondary_direction: ItemDirection,
    primary_scrollbar: Behaviour,
    secondary_scrollbar: Behaviour,
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
            width: Length::Fill,
            height: Length::Fill,
            max_width: f32::MAX,
            max_height: f32::MAX,
            item_width: 0.,
            item_height: 0.,
            max_wrap: u32::MAX,
            min_wrap: 1,
            line_scroll: 0.,
            primary_direction: ItemDirection::LeftToRight,
            secondary_direction: ItemDirection::TopToBottom,
            primary_scrollbar: Behaviour::Enabled,
            secondary_scrollbar: Behaviour::Disabled,
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
        if matches!(event, iced_native::Event::Mouse(_))
            && !layout.bounds().contains(cursor_position)
        {
            return iced_native::event::Status::Ignored;
        }

        let state = tree.state.downcast_mut::<State>();

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
        state.offset_y = state
            .offset_y
            .min(content_size.height - view_size.height)
            .max(0.);

        let (first, last) = self.visible_pos(view_size, state);
        if first_o != first || last_o != last {
            shell.invalidate_layout();
        }

        self.state = Some(*state);

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
        style: &iced_native::renderer::Style,
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
        Size::new(
            size.width - self.padding.left - self.padding.right,
            size.height - self.padding.top - self.padding.bottom,
        )
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

pub enum Behaviour {
    Enabled,
    Disabled,
    Hidden,
}

pub struct WrapBoxStyle {
    background: Background,
    border_color: Color,
    border_radius: BorderRadius,
    border_thickness: f32,
    scrollbar_border_radius: BorderRadius,
    scrollbar_border_thickness: f32,
    scrollbar_border_color: Color,
    scrollbar_show_buttons: bool,
    scrollbar_buttons_foreground: Color,
    scrollbar_buttons_background: Background,
    scrollbar_buttons_hover_foreground: Color,
    scrollbar_buttons_hover_background: Background,
    scrollbar_buttons_pressed_foreground: Color,
    scrollbar_buttons_pressed_background: Background,
    scrollbar_buttons_inactive_foreground: Color,
    scrollbar_buttons_inactive_background: Background,
    scrollbar_buttons_inactive_hover_foreground: Color,
    scrollbar_buttons_inactive_hover_background: Background,
    scrollbar_buttons_inactive_pressed_foreground: Color,
    scrollbar_buttons_inactive_pressed_background: Background,
    scrollbar_topleft_trough: Background,
    scrollbar_bottomright_trough: Background,
    scrollbar_topleft_trough_hover: Background,
    scrollbar_bottomright_trough_hover: Background,
    scrollbar_thumb_border_radius: BorderRadius,
    scrollbar_thumb_border_thickness: f32,
    scrollbar_thumb_border_color: Color,
    scrollbar_thumb_color: Background,
    scrollbar_thumb_hover_color: Background,
    scrollbar_thumb_hover_border_radius: BorderRadius,
    scrollbar_thumb_hover_border_thickness: f32,
    scrollbar_thumb_hover_border_color: Color,
    scrollbar_thumb_pressed_color: Background,
    scrollbar_thumb_pressed_border_radius: BorderRadius,
    scrollbar_thumb_pressed_border_thickness: f32,
    scrollbar_thumb_pressed_border_color: Color,
}

impl WrapBoxStyle {
    pub fn dark() -> Self {
        Self {
            background: Background::Color(Color::TRANSPARENT),
            border_color: Color::BLACK,
            border_radius: BorderRadius::default(),
            border_thickness: 0.,
            scrollbar_border_radius: BorderRadius::default(),
            scrollbar_border_thickness: 0.,
            scrollbar_border_color: Color::BLACK,
            scrollbar_show_buttons: true,
            scrollbar_buttons_foreground: Color::from_rgb8(0xEE, 0xEE, 0xEE),
            scrollbar_buttons_background: Background::Color(Color::from_rgb8(
                0x22, 0x22, 0x22,
            )),
            scrollbar_buttons_hover_foreground: Color::from_rgb8(
                0xEE, 0xEE, 0xEE,
            ),
            scrollbar_buttons_hover_background: Background::Color(
                Color::from_rgb8(0x33, 0x33, 0x33),
            ),
            scrollbar_buttons_pressed_foreground: Color::from_rgb8(
                0xEE, 0xEE, 0xEE,
            ),
            scrollbar_buttons_pressed_background: Background::Color(
                Color::from_rgb8(0x18, 0x18, 0x18),
            ),
            scrollbar_buttons_inactive_foreground: Color::from_rgb8(
                0x77, 0x77, 0x77,
            ),
            scrollbar_buttons_inactive_background: Background::Color(
                Color::from_rgb8(0x22, 0x22, 0x22),
            ),
            scrollbar_buttons_inactive_hover_foreground: Color::from_rgb8(
                0x77, 0x77, 0x77,
            ),
            scrollbar_buttons_inactive_hover_background: Background::Color(
                Color::from_rgb8(0x22, 0x22, 0x22),
            ),
            scrollbar_buttons_inactive_pressed_foreground: Color::from_rgb8(
                0x77, 0x77, 0x77,
            ),
            scrollbar_buttons_inactive_pressed_background: Background::Color(
                Color::from_rgb8(0x22, 0x22, 0x22),
            ),
            scrollbar_topleft_trough: Background::Color(Color::from_rgb8(
                0x18, 0x18, 0x18,
            )),
            scrollbar_bottomright_trough: Background::Color(Color::from_rgb8(
                0x18, 0x18, 0x18,
            )),
            scrollbar_topleft_trough_hover: Background::Color(
                Color::from_rgb8(0x22, 0x22, 0x22),
            ),
            scrollbar_bottomright_trough_hover: Background::Color(
                Color::from_rgb8(0x22, 0x22, 0x22),
            ),
            scrollbar_thumb_border_radius: BorderRadius::default(),
            scrollbar_thumb_border_thickness: 0.,
            scrollbar_thumb_border_color: Color::BLACK,
            scrollbar_thumb_color: Background::Color(Color::from_rgb8(
                0x33, 0x33, 0x33,
            )),
            scrollbar_thumb_hover_border_radius: BorderRadius::default(),
            scrollbar_thumb_hover_border_thickness: 0.,
            scrollbar_thumb_hover_border_color: Color::BLACK,
            scrollbar_thumb_hover_color: Background::Color(Color::from_rgb8(
                0x55, 0x55, 0x55,
            )),
            scrollbar_thumb_pressed_border_radius: BorderRadius::default(),
            scrollbar_thumb_pressed_border_thickness: 0.,
            scrollbar_thumb_pressed_border_color: Color::BLACK,
            scrollbar_thumb_pressed_color: Background::Color(
                Color::from_rgb8(0x22, 0x22, 0x22),
            ),
        }
    }
}
