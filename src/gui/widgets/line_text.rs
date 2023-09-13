use std::borrow::Cow;

use iced_core::{
    alignment::{Horizontal, Vertical},
    layout::Node,
    text::{self, Shaping},
    Color, Element, Length, Pixels, Text, Widget,
};

use super::{limit_size, sides::Sides};

/// Text that will not wrap
pub struct LineText<'a, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    content: Cow<'a, str>,
    size: Option<f32>,
    width: Length,
    height: Length,
    padding: Sides<f32>,
    horizontal_alignment: Horizontal,
    vertical_alignment: Vertical,
    font: Option<Renderer::Font>,
    shaping: Shaping,
    elipsis: Cow<'a, str>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Renderer> LineText<'a, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    pub fn new<S>(content: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        LineText {
            content: content.into(),
            size: None,
            font: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: 0.into(),
            horizontal_alignment: Horizontal::Left,
            vertical_alignment: Vertical::Top,
            shaping: Shaping::Basic,
            elipsis: "".into(),
            style: Default::default(),
        }
    }

    pub fn size<S>(mut self, size: S) -> Self
    where
        S: Into<Pixels>,
    {
        self.size = Some(size.into().0);
        self
    }

    pub fn font<F>(mut self, font: F) -> Self
    where
        F: Into<Renderer::Font>,
    {
        self.font = Some(font.into());
        self
    }

    pub fn style(
        mut self,
        style: <Renderer::Theme as StyleSheet>::Style,
    ) -> Self {
        self.style = style;
        self
    }

    /// Sets the width of the [`Text`] boundaries.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn padding(mut self, padding: impl Into<Sides<f32>>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn horizontal_alignment(mut self, alignment: Horizontal) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    pub fn vertical_alignment(mut self, alignment: Vertical) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    pub fn shaping(mut self, shaping: Shaping) -> Self {
        self.shaping = shaping;
        self
    }

    pub fn elipsis<S>(mut self, elipsis: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.elipsis = elipsis.into();
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for LineText<'a, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &iced_core::layout::Limits,
    ) -> Node {
        let limits = limits
            .width(self.width)
            .height(self.height)
            .pad(self.padding.into());

        let font_size = self.size.unwrap_or_else(|| renderer.default_size());
        let font = self.font.unwrap_or_else(|| renderer.default_font());

        let width = renderer.measure_width(
            &self.content,
            font_size,
            font,
            self.shaping,
        ) + 1.;

        let lim = limits.min_width(width).min_height(font_size);

        Node::new(limit_size(&lim, self.width, self.height))
    }

    fn draw(
        &self,
        _state: &iced_core::widget::Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced_core::Renderer>::Theme,
        style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        _cursor: iced_core::mouse::Cursor,
        _viewport: &iced_core::Rectangle,
    ) {
        let mut bounds = layout.bounds();

        bounds.x = match self.horizontal_alignment {
            Horizontal::Left => bounds.x,
            Horizontal::Center => bounds.center_x(),
            Horizontal::Right => bounds.x + bounds.width,
        };

        bounds.y = match self.vertical_alignment {
            Vertical::Top => bounds.y,
            Vertical::Center => bounds.center_y(),
            Vertical::Bottom => bounds.y + bounds.height,
        };

        let font_size = self.size.unwrap_or_else(|| renderer.default_size());
        let font = self.font.unwrap_or_else(|| renderer.default_font());

        let text_width = renderer.measure_width(
            &self.content,
            font_size,
            font,
            self.shaping,
        );

        if text_width < bounds.width {
            renderer.fill_text(Text {
                content: &self.content,
                size: font_size,
                bounds,
                line_height: Default::default(),
                color: theme
                    .foreground(&self.style)
                    .unwrap_or(style.text_color),
                font,
                horizontal_alignment: self.horizontal_alignment,
                vertical_alignment: self.vertical_alignment,
                shaping: self.shaping,
            });
            return;
        }

        let elipsis_width = renderer.measure_width(
            &self.elipsis,
            font_size,
            font,
            self.shaping,
        );

        let width = bounds.width - elipsis_width;

        // Hit test is bugged and doesn't work
        /*let hit = renderer.hit_test(
            &self.content,
            font_size,
            Default::default(),
            font,
            Size::new(f32::MAX, f32::MAX),
            self.shaping,
            Point::new(width, font_size),
            false,
        );

        let hit = if let Some(Hit::CharOffset(hit)) = hit {
            hit
        } else {
            // The content is empty
            return;
        };*/

        // this is not the number of characters, but approximation
        let str_len = self.content.as_bytes().len();
        // As a workaround, assume that the font is monospace
        // and subtract 3 so that we can be sure that the resulting
        // string will be shorter
        let hit = (((str_len as f32 * width) / text_width) as usize)
            .checked_sub(3)
            .unwrap_or(0);

        let mut content = String::new();
        content.extend(self.content.chars().take(hit));
        content += &self.elipsis;

        renderer.fill_text(Text {
            content: &content,
            size: font_size,
            bounds,
            line_height: Default::default(),
            color: theme.foreground(&self.style).unwrap_or(style.text_color),
            font,
            horizontal_alignment: self.horizontal_alignment,
            vertical_alignment: self.vertical_alignment,
            shaping: self.shaping,
        });
    }
}

impl<'a, Message, Renderer> From<LineText<'a, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: text::Renderer + 'a,
    Renderer::Theme: StyleSheet,
{
    fn from(text: LineText<'a, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(text)
    }
}

pub trait StyleSheet {
    type Style: Default;

    fn foreground(&self, style: &Self::Style) -> Option<Color> {
        _ = style;
        None
    }
}
