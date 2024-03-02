use std::ops::{Add, Range, Sub};

use iced_core::{
    event::Status,
    layout::{Limits, Node},
    mouse::Interaction,
    overlay::Group,
    widget::Tree,
    Element, Length, Size, Vector, Widget,
};
use itertools::Itertools;

use super::sides::Sides;

/// Container that can separate its contents into grid and allows spanning
pub struct Grid<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    width: Length,
    height: Length,
    /// Guaranteed to have at least two items
    columns: Vec<SpanSum>,
    /// Guaranteed to have at least two items
    rows: Vec<SpanSum>,
    items: Vec<GridItem<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Grid<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    pub fn new<I1, I2>(
        column: I1,
        row: I2,
        items: Vec<GridItem<'a, Message, Theme, Renderer>>,
    ) -> Self
    where
        I1: Iterator<Item = SpanLen>,
        I2: Iterator<Item = SpanLen>,
    {
        let mut columns = vec![SpanSum::new(0., 0.)];
        let mut rows = vec![SpanSum::new(0., 0.)];

        columns.extend(column.scan(SpanSum::new(0., 0.), |cs, c| {
            match c {
                SpanLen::Fixed(v) => cs.fixed += v,
                SpanLen::Relative(v) => cs.relative += v,
            }

            Some(*cs)
        }));
        rows.extend(row.scan(SpanSum::new(0., 0.), |rs, r| {
            match r {
                SpanLen::Fixed(v) => rs.fixed += v,
                SpanLen::Relative(v) => rs.relative += v,
            }

            Some(*rs)
        }));

        if columns.len() == 1 {
            columns.push(SpanSum::new(0., 1.))
        }
        if rows.len() == 1 {
            rows.push(SpanSum::new(0., 1.))
        }

        Self {
            width: Length::Fill,
            height: Length::Fill,
            columns,
            rows,
            items,
        }
    }

    pub fn width<L>(mut self, width: L) -> Self
    where
        L: Into<Length>,
    {
        self.width = width.into();
        self
    }

    pub fn height<L>(mut self, height: L) -> Self
    where
        L: Into<Length>,
    {
        self.height = height.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Grid<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        self.items.iter().map(|i| Tree::new(&i.child)).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        for (i, t) in self.items.iter().zip(&mut tree.children) {
            t.diff(&i.child)
        }
    }

    fn size(&self) -> Size<Length> {
        Size { width: self.width, height: self.height }
    }

    fn layout(&self, state: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let size = limits.width(self.width).height(self.height).max();

        let c_point = ((size.width - self.col_last().fixed)
            / self.col_last().relative)
            .max(0.);
        let r_point = ((size.height - self.row_last().fixed)
            / self.row_last().relative)
            .max(0.);

        let mut children = Vec::new();

        for (i, state) in self.items.iter().zip(state.children.iter_mut()) {
            let w = (self.columns[i.columns.end]
                - self.columns[i.columns.start])
                .with_unit(c_point)
                - i.padding.lr_sum();

            let h = (self.rows[i.rows.end] - self.rows[i.rows.start])
                .with_unit(r_point)
                - i.padding.tb_sum();

            let x = self.columns[i.columns.start].with_unit(c_point)
                + i.padding.left;
            let y = self.rows[i.rows.start].with_unit(c_point) + i.padding.top;

            let child_limits = Limits::new(Size::ZERO, Size::new(w, h));
            let child = i.child.as_widget().layout(state, renderer, &child_limits);

            children.push(child.translate(Vector::new(x, y)));
        }

        Node::with_children(size, children)
    }

    fn operate(
        &self,
        state: &mut Tree,
        layout: iced_core::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced_core::widget::Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.iter()
                .zip(layout.children())
                .zip(state.children.iter_mut())
                .for_each(|((child, layout), state)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation)
                })
        })
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: iced_core::Event,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced_core::Clipboard,
        shell: &mut iced_core::Shell<'_, Message>,
        viewport: &iced_core::Rectangle,
    ) -> Status {
        self.iter_mut()
            .zip(layout.children())
            .zip(state.children.iter_mut())
            .map(|((child, layout), state)| {
                child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(Status::Ignored, |c, s| s.merge(c))
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced_core::Rectangle,
        renderer: &Renderer,
    ) -> Interaction {
        self.iter()
            .zip(layout.children())
            .zip(state.children.iter())
            .map(|((child, layout), state)| {
                child.as_widget().mouse_interaction(
                    state, layout, cursor, viewport, renderer,
                )
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        state: &iced_core::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced_core::Rectangle,
    ) {
        self.iter()
            .zip(layout.children())
            .zip(state.children.iter())
            .for_each(|((child, layout), state)| {
                child.as_widget().draw(
                    state, renderer, theme, style, layout, cursor, viewport,
                )
            })
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: iced_core::Layout<'_>,
        renderer: &Renderer,
        transpation: Vector,
    ) -> Option<iced_core::overlay::Element<'b, Message, Theme, Renderer>> {
        let children = self
            .iter_mut()
            .zip(layout.children())
            .zip(state.children.iter_mut())
            .filter_map(|((child, layout), state)| {
                child.as_widget_mut().overlay(state, layout, renderer, transpation)
            })
            .collect_vec();

        (!children.is_empty())
            .then(|| Group::with_children(children).overlay())
    }
}

impl<'a, Message, Theme, Renderer> From<Grid<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
    Theme: 'a,
    Message: 'a,
{
    fn from(value: Grid<'a, Message, Theme, Renderer>) -> Self {
        Self::new(value)
    }
}

pub struct GridItem<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    columns: Range<usize>,
    rows: Range<usize>,
    padding: Sides<f32>,
    child: Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> GridItem<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    pub fn new<E>(item: E) -> Self
    where
        E: Into<Element<'a, Message, Theme, Renderer>>,
    {
        Self {
            columns: 0..1,
            rows: 0..1,
            padding: 0.into(),
            child: item.into(),
        }
    }

    pub fn column<P>(mut self, column: P) -> Self
    where
        P: Into<GridPosition>,
    {
        self.columns = column.into().0;
        self
    }

    pub fn row<P>(mut self, row: P) -> Self
    where
        P: Into<GridPosition>,
    {
        self.rows = row.into().0;
        self
    }

    pub fn padding<P>(mut self, padding: P) -> Self
    where
        P: Into<Sides<f32>>,
    {
        self.padding = padding.into();
        self
    }
}

impl<'a, Message, Theme, Renderer, I> From<I> for GridItem<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
    I: Into<Element<'a, Message, Theme, Renderer>>,
{
    fn from(value: I) -> Self {
        Self::new(value)
    }
}

pub struct GridPosition(Range<usize>);

impl From<usize> for GridPosition {
    fn from(value: usize) -> Self {
        GridPosition(value..value + 1)
    }
}

impl From<Range<usize>> for GridPosition {
    fn from(value: Range<usize>) -> Self {
        GridPosition(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SpanLen {
    Fixed(f32),
    Relative(f32),
}

impl<'a, Message, Theme, Renderer> Grid<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    fn col_last(&self) -> SpanSum {
        // columns is guaranteed to have at least one item
        *self.columns.last().unwrap()
    }

    fn row_last(&self) -> SpanSum {
        // rows is guarenteed to have at least one item
        *self.rows.last().unwrap()
    }

    fn iter(&self) -> impl Iterator<Item = &Element<'a, Message, Theme, Renderer>> {
        self.items.iter().map(|i| &i.child)
    }

    fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut Element<'a, Message, Theme, Renderer>> {
        self.items.iter_mut().map(|i| &mut i.child)
    }
}

#[derive(Clone, Copy, Debug)]
struct SpanSum {
    /// Total fixed so far including this
    fixed: f32,
    /// Total relative so far including this
    relative: f32,
}

impl SpanSum {
    fn new(fixed: f32, relative: f32) -> Self {
        Self { fixed, relative }
    }

    /// Calculates fixed size with the given size per relative unit: `pt`
    fn with_unit(&self, pt: f32) -> f32 {
        self.fixed + self.relative * pt
    }
}

impl Add for SpanSum {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.fixed + rhs.fixed, self.relative + rhs.relative)
    }
}

impl Sub for SpanSum {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.fixed - rhs.fixed, self.relative - rhs.relative)
    }
}
