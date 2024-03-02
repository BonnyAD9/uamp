use iced::{
    application,
    overlay::menu,
    widget::{
        button, checkbox, container, pane_grid, pick_list, progress_bar,
        radio, rule, scrollable, slider, svg, text, text_input, toggler,
    },
};
use iced_core::{
    gradient::{ColorStop, Linear},
    Background, Color, Degrees, Gradient, Size, Vector,
};

use super::widgets::{
    border, cursor_grad, line_text, sides::Sides, svg_button, switch, wrap_box, NO_SHADOW,
};

/// Creates const color from hex
///
/// # Example
/// ```
/// const col: Color = const_color!(0x123456);
/// ```
macro_rules! const_color {
    ($x:literal) => {
        Color::from_rgb(
            (($x & 0xFF0000) >> 16) as f32 / 255.,
            (($x & 0xFF00) >> 8) as f32 / 255.,
            ($x & 0xFF) as f32 / 255.,
        )
    };
}

// some default colors to use

const TRANSPARENT: Color = Color::TRANSPARENT;
const TRANSPARENT_BG: Background = Background::Color(TRANSPARENT);
const BG_BRIGHT: Color = const_color!(0x222222);
const BG_BRIGHT_BG: Background = Background::Color(BG_BRIGHT);
const BG_BRIGHTER: Color = const_color!(0x282828);
const BG_BRIGHTER_BG: Background = Background::Color(BG_BRIGHTER);
const BG_GRAY: Color = const_color!(0x1E1E1E);
const BG_GRAY_BG: Background = Background::Color(BG_GRAY);
const BG_DARK: Color = const_color!(0x181818);
const BG_DARK_BG: Background = Background::Color(BG_DARK);
const BG_BRIGHT_RED: Color = const_color!(0x361818);
const BG_BRIGHT_RED_BG: Background = Background::Color(BG_BRIGHT_RED);
/// The outline color
const OUTLINE: Color = const_color!(0x555555);
/// The outline color as background
const OUTLINE_BG: Background = Background::Color(OUTLINE);
/// The outline color
const GRAY_OUTLINE: Color = const_color!(0x444444);
/// The outline color as background
const GRAY_OUTLINE_BG: Background = Background::Color(GRAY_OUTLINE);
/// The outline color
const DARK_OUTLINE: Color = const_color!(0x333333);
/// The outline color as background
const DARK_OUTLINE_BG: Background = Background::Color(DARK_OUTLINE);
/// The foreground color
const FOREGROUND: Color = const_color!(0xEEEEEE);
const FOREGROUND_BG: Background = Background::Color(FOREGROUND);
/// The inactice foreground color
const GRAY_FG: Color = const_color!(0x777777);
/// The color of pressed button
const PRESSED: Color = const_color!(0x333333);
/// The color of pressed button as background
const PRESSED_BG: Background = Background::Color(PRESSED);
/// The color of selected button
const SELECTED: Color = const_color!(0x444444);
/// The color of selected button as background
const SELECTED_BG: Background = Background::Color(SELECTED);
/// The contras color
const CONTRAST: Color = const_color!(0xEEEE55);
/// The contras color as background
const CONTRAST_BG: Background = Background::Color(CONTRAST);
/// The contras color
const DARK_CONTRAST: Color = const_color!(0xBBAA22);
/// The contras color as background
const DARK_CONTRAST_BG: Background = Background::Color(DARK_CONTRAST);
/// Brighter version of contrast color
const BRIGHT_CONTRAST: Color = const_color!(0xEEEE33);
/// The contras color as background
const BRIGHT_CONTRAST_BG: Background = Background::Color(BRIGHT_CONTRAST);
/// The border radius
const RADIUS: f32 = 6.;
const RED: Color = const_color!(0xEE5555);
const BRIGHT_RED: Color = const_color!(0xEE3333);

/// The theme of uamp app
#[derive(Default, Clone)]
pub struct Theme {}

impl application::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> application::Appearance {
        application::Appearance {
            background_color: BG_DARK,
            text_color: FOREGROUND,
        }
    }
}

/// The types of styles of buttons
#[derive(Default, PartialEq)]
pub enum Button {
    /// Default button style
    #[default]
    Default,
    GrayHover,
    SelectedGrayHover,
}

impl button::StyleSheet for Theme {
    type Style = Button;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let default = button::Appearance {
            shadow_offset: Vector::ZERO,
            background: None,
            border: iced::Border {
                radius: RADIUS.into(),
                width: 0.,
                color: TRANSPARENT,
            },
            text_color: FOREGROUND,
            shadow: NO_SHADOW,
        };

        match style {
            Button::GrayHover => button::Appearance {
                text_color: GRAY_FG,
                ..default
            },
            Button::SelectedGrayHover => button::Appearance {
                text_color: DARK_CONTRAST,
                ..default
            },
            _ => default,
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let base = button::Appearance {
            text_color: CONTRAST,
            ..self.active(style)
        };
        match style {
            Button::GrayHover | Button::SelectedGrayHover => {
                button::Appearance {
                    text_color: CONTRAST,
                    ..base
                }
            }
            _ => base,
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        let base = button::Appearance {
            text_color: BRIGHT_CONTRAST,
            ..self.active(style)
        };

        match style {
            Button::GrayHover | Button::SelectedGrayHover => {
                button::Appearance {
                    text_color: BRIGHT_CONTRAST,
                    ..base
                }
            }
            _ => base,
        }
    }

    // fn disabled(&self, style: &Self::Style) -> button::Appearance;
}

impl checkbox::StyleSheet for Theme {
    type Style = ();

    fn active(
        &self,
        _style: &Self::Style,
        is_checked: bool,
    ) -> checkbox::Appearance {
        checkbox::Appearance {
            background: BG_DARK_BG,
            icon_color: CONTRAST,
            border: iced::Border {
                radius: RADIUS.into(),
                width: 0.,
                color: OUTLINE,
            },
            text_color: if is_checked { Some(CONTRAST) } else { None },
        }
    }

    fn hovered(
        &self,
        style: &Self::Style,
        is_checked: bool,
    ) -> checkbox::Appearance {
        checkbox::Appearance {
            background: SELECTED_BG,
            ..self.active(style, is_checked)
        }
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub enum Container {
    #[default]
    Default,
    Gray,
    Dark,
    TopGrad,
    Float,
}

impl container::StyleSheet for Theme {
    type Style = Container;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let default = container::Appearance {
            ..container::Appearance::default()
        };

        match style {
            Container::Gray => container::Appearance {
                background: Some(BG_GRAY_BG),
                ..default
            },
            Container::TopGrad => container::Appearance {
                background: Some(Background::Gradient(Gradient::Linear(
                    Linear::new(Degrees(270.)).add_stops([
                        ColorStop {
                            offset: 0.,
                            color: BG_GRAY,
                        },
                        ColorStop {
                            offset: 0.8,
                            color: BG_DARK,
                        },
                    ]),
                ))),
                ..default
            },
            Container::Dark => container::Appearance {
                background: Some(BG_DARK_BG),
                ..default
            },
            Container::Float => container::Appearance {
                background: Some(BG_BRIGHT_BG),
                border: iced::Border {
                    radius: 6.0.into(),
                    ..default.border
                },
                ..default
            },
            _ => default,
        }
    }
}

impl slider::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            rail: slider::Rail {
                colors: (CONTRAST, OUTLINE),
                width: 4.,
                border_radius: 2.0.into(),
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 0. },
                color: TRANSPARENT,
                border_width: 0.,
                border_color: OUTLINE,
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> slider::Appearance {
        let base = self.active(style);
        slider::Appearance {
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 5. },
                color: FOREGROUND,
                ..base.handle
            },
            ..base
        }
    }

    fn dragging(&self, style: &Self::Style) -> slider::Appearance {
        let base = self.active(style);
        slider::Appearance {
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 5. },
                color: DARK_CONTRAST,
                border_color: CONTRAST,
                ..base.handle
            },
            ..base
        }
    }
}

impl menu::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> menu::Appearance {
        menu::Appearance {
            text_color: FOREGROUND,
            background: BG_GRAY_BG,
            border: iced::Border {
                width: 0.,
                radius: RADIUS.into(),
                color: OUTLINE,
            },
            selected_text_color: CONTRAST,
            selected_background: SELECTED_BG,
        }
    }
}

impl pick_list::StyleSheet for Theme {
    type Style = ();

    fn active(
        &self,
        _style: &<Self as pick_list::StyleSheet>::Style,
    ) -> pick_list::Appearance {
        pick_list::Appearance {
            text_color: FOREGROUND,
            placeholder_color: GRAY_FG,
            handle_color: CONTRAST,
            background: BG_DARK_BG,
            border: iced::Border {
                radius: RADIUS.into(),
                width: 0.,
                color: OUTLINE,
            },
        }
    }

    fn hovered(
        &self,
        style: &<Self as pick_list::StyleSheet>::Style,
    ) -> pick_list::Appearance {
        pick_list::Appearance {
            background: SELECTED_BG,
            ..self.active(style)
        }
    }
}

impl radio::StyleSheet for Theme {
    type Style = ();

    fn active(
        &self,
        _style: &Self::Style,
        is_selected: bool,
    ) -> radio::Appearance {
        radio::Appearance {
            background: BG_DARK_BG,
            dot_color: if is_selected { CONTRAST } else { SELECTED },
            border_width: 0.,
            border_color: OUTLINE,
            text_color: if is_selected { Some(FOREGROUND) } else { None },
        }
    }

    fn hovered(
        &self,
        style: &Self::Style,
        is_selected: bool,
    ) -> radio::Appearance {
        radio::Appearance {
            background: SELECTED_BG,
            ..self.active(style, is_selected)
        }
    }
}

impl toggler::StyleSheet for Theme {
    type Style = ();

    fn active(
        &self,
        _style: &Self::Style,
        is_active: bool,
    ) -> toggler::Appearance {
        toggler::Appearance {
            background: if is_active { DARK_CONTRAST } else { OUTLINE },
            background_border_width: 0.,
            background_border_color: TRANSPARENT,
            foreground: FOREGROUND,
            foreground_border_width: 0.,
            foreground_border_color: TRANSPARENT,
        }
    }

    fn hovered(
        &self,
        style: &Self::Style,
        is_active: bool,
    ) -> toggler::Appearance {
        toggler::Appearance {
            foreground: CONTRAST,
            ..self.active(style, is_active)
        }
    }
}

impl pane_grid::StyleSheet for Theme {
    type Style = ();

    fn picked_split(&self, _style: &Self::Style) -> Option<pane_grid::Line> {
        Some(pane_grid::Line {
            color: OUTLINE,
            width: 0.,
        })
    }

    fn hovered_split(&self, _style: &Self::Style) -> Option<pane_grid::Line> {
        Some(pane_grid::Line {
            color: CONTRAST,
            width: 0.,
        })
    }

    fn hovered_region(&self, _style: &Self::Style) -> pane_grid::Appearance {
        pane_grid::Appearance {
            background: SELECTED_BG,
            border: iced::Border {
                width: 0.,
                color: OUTLINE,
                radius: RADIUS.into(),
            }
        }
    }
}

impl progress_bar::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> progress_bar::Appearance {
        progress_bar::Appearance {
            background: BG_DARK_BG,
            bar: CONTRAST_BG,
            border_radius: RADIUS.into(),
        }
    }
}

impl rule::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> rule::Appearance {
        rule::Appearance {
            color: SELECTED,
            width: 4,
            radius: RADIUS.into(),
            fill_mode: rule::FillMode::Full,
        }
    }
}

#[derive(Default)]
pub enum Svg {
    /// Original svg color
    #[default]
    Original,
}

impl svg::StyleSheet for Theme {
    type Style = Svg;

    fn appearance(&self, style: &Self::Style) -> svg::Appearance {
        match style {
            _ => svg::Appearance::default(),
        }
    }
}

impl scrollable::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> scrollable::Appearance {
        scrollable::Appearance {
            container: container::Appearance {
                text_color: None,
                background: None,
                border: iced::Border {
                    radius: RADIUS.into(),
                    width: 0.,
                    color: OUTLINE,
                },
                shadow: NO_SHADOW,
            },
            scrollbar: scrollable::Scrollbar {
                background: None,
                border: iced::Border {
                    radius: RADIUS.into(),
                    width: 0.,
                    color: OUTLINE,
                },
                scroller: scrollable::Scroller {
                    color: BG_GRAY,
                    border: iced::Border {
                        radius: RADIUS.into(),
                        width: 0.,
                        color: OUTLINE,
                    }
                },
            },
            gap: None,
        }
    }

    fn hovered(
        &self,
        style: &Self::Style,
        is_mouse_over_scrollbar: bool,
    ) -> scrollable::Appearance {
        let base = self.active(style);
        scrollable::Appearance {
            scrollbar: scrollable::Scrollbar {
                scroller: scrollable::Scroller {
                    color: if is_mouse_over_scrollbar {
                        SELECTED
                    } else {
                        BG_GRAY
                    },
                    ..base.scrollbar.scroller
                },
                ..base.scrollbar
            },
            ..base
        }
    }

    // fn dragging(&self, style: &Self::Style) -> scrollable::Scrollbar;
    // fn active_horizontal(
    //     &self,
    //     style: &Self::Style
    // ) -> scrollable::Scrollbar;
    // fn hovered_horizontal(
    //     &self,
    //     style: &Self::Style,
    //     is_mouse_over_scrollbar: bool,
    // ) -> scrollable::Scrollbar;
    // fn dragging_horizontal(
    //     &self,
    //     style: &Self::Style,
    // ) -> scrollable::Scrollbar;
}

/// The text styles
#[derive(Clone, Default, Copy)]
pub enum Text {
    /// The default text style
    #[default]
    Default,
    NoForeground,
    Contrast,
    Gray,
}

impl text::StyleSheet for Theme {
    type Style = Text;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
        text::Appearance {
            color: match style {
                Text::Default => Some(FOREGROUND),
                Text::NoForeground => None,
                Text::Contrast => Some(CONTRAST),
                Text::Gray => Some(GRAY_FG),
            },
        }
    }
}

#[derive(Default)]
pub enum TextInput {
    #[default]
    Default,
    Invalid,
}

impl text_input::StyleSheet for Theme {
    type Style = TextInput;

    fn active(&self, style: &Self::Style) -> text_input::Appearance {
        let default = text_input::Appearance {
            background: TRANSPARENT_BG,
            border: iced::Border {
                radius: RADIUS.into(),
                width: 0.,
                color: OUTLINE,
            },
            icon_color: BG_GRAY,
        };

        match style {
            TextInput::Invalid => text_input::Appearance {
                background: BG_BRIGHT_RED_BG,
                ..default
            },
            _ => default,
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        let base = self.active(style);
        match style {
            TextInput::Default => text_input::Appearance {
                background: BG_BRIGHT_BG,
                ..base
            },
            _ => base,
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        GRAY_FG
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        FOREGROUND
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        FOREGROUND
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        DARK_CONTRAST
    }

    fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
        self.active(style)
    }

    // fn disabled_color(&self, style: &Self::Style) -> Color;
}

#[derive(Default, Clone)]
pub enum WrapBox {
    #[default]
    Dark,
    Bright,
}

impl wrap_box::StyleSheet for Theme {
    type Style = WrapBox;

    fn background(
        &self,
        style: &Self::Style,
        _pos: wrap_box::MousePos,
    ) -> wrap_box::SquareStyle {
        let base = wrap_box::SquareStyle {
            background: BG_DARK_BG,
            border: TRANSPARENT,
            border_thickness: 0.,
            border_radius: 0.0.into(),
        };

        match style {
            WrapBox::Bright => wrap_box::SquareStyle {
                background: TRANSPARENT_BG,
                ..base
            },
            _ => base,
        }
    }

    fn button_style(
        &self,
        _style: &Self::Style,
        pos: wrap_box::MousePos,
        pressed: bool,
        is_start: bool,
        relative_scroll: f32,
    ) -> wrap_box::ButtonStyle {
        let square = wrap_box::SquareStyle {
            background: TRANSPARENT_BG,
            border: TRANSPARENT,
            border_thickness: 0.0.into(),
            border_radius: RADIUS.into(),
        };

        if is_start && relative_scroll == 0.
            || !is_start && relative_scroll == 1.
        {
            // inactive
            wrap_box::ButtonStyle {
                square: wrap_box::SquareStyle {
                    border_thickness: 0.,
                    ..square
                },
                foreground: GRAY_FG,
            }
        } else {
            // active

            let foreground = if pressed {
                BRIGHT_CONTRAST
            } else if pos == wrap_box::MousePos::DirectlyOver {
                CONTRAST
            } else {
                FOREGROUND
            };

            wrap_box::ButtonStyle { square, foreground }
        }
    }

    fn thumb_style(
        &self,
        style: &Self::Style,
        pos: wrap_box::MousePos,
        pressed: bool,
        _relative_scroll: f32,
    ) -> wrap_box::SquareStyle {
        let mut square = wrap_box::SquareStyle {
            background: BG_BRIGHT_BG,
            border: TRANSPARENT,
            border_thickness: 0.,
            border_radius: RADIUS.into(),
        };

        square = match style {
            WrapBox::Bright => wrap_box::SquareStyle {
                background: BG_BRIGHTER_BG,
                ..square
            },
            _ => square,
        };

        if pressed {
            wrap_box::SquareStyle {
                background: SELECTED_BG,
                ..square
            }
        } else if pos == wrap_box::MousePos::DirectlyOver {
            wrap_box::SquareStyle {
                background: PRESSED_BG,
                ..square
            }
        } else {
            square
        }
    }

    fn trough_style(
        &self,
        _style: &Self::Style,
        _pos: wrap_box::MousePos,
        is_start: bool,
        _relative_scroll: f32,
    ) -> wrap_box::SquareStyle {
        wrap_box::SquareStyle {
            background: TRANSPARENT_BG,
            border: PRESSED,
            border_thickness: 0.0,
            border_radius: if is_start {
                [RADIUS, RADIUS, 0., 0.].into()
            } else {
                [0., 0., RADIUS, RADIUS].into()
            },
        }
    }
}

impl wrap_box::LayoutStyleSheet<()> for Theme {
    fn layout(&self, _style: &()) -> wrap_box::LayoutStyle {
        wrap_box::LayoutStyle {
            padding: Some([0, 0, 0, 20].into()),
            spacing: (None, Some(1.)),
            ..wrap_box::LayoutStyle::default()
        }
    }
}

#[derive(Default)]
pub enum Border {
    #[default]
    None,
    SongItem,
    Bot,
    LeftRound(bool),
    BotRound(bool),
}

impl border::StyleSheet for Theme {
    type Style = Border;

    fn background(&self, style: &Self::Style) -> Background {
        match style {
            Border::None => TRANSPARENT_BG,
            _ => TRANSPARENT_BG,
        }
    }

    fn border_thickness(&self, style: &Self::Style) -> Sides<f32> {
        match style {
            Border::Bot => [0, 0, 2, 0].into(),
            Border::LeftRound(true) => [0, 0, 0, 4].into(),
            Border::BotRound(true) => [0, 0, 4, 0].into(),
            Border::SongItem => [1, 0, 0, 0].into(),
            _ => 0.into(),
        }
    }

    fn border_radius(&self, style: &Self::Style) -> Sides<f32> {
        match style {
            Border::Bot => 15.into(),
            Border::SongItem => 6.into(),
            _ => 0.into(),
        }
    }

    fn border_color(&self, style: &Self::Style) -> Sides<Background> {
        match style {
            Border::None => OUTLINE_BG.into(),
            Border::Bot => DARK_OUTLINE_BG.into(),
            Border::SongItem => GRAY_OUTLINE_BG.into(),
            Border::LeftRound(_) | Border::BotRound(_) => CONTRAST_BG.into(),
        }
    }

    fn corner_color(&self, _style: &Self::Style) -> Sides<Color> {
        OUTLINE.into()
    }

    fn border_border_radius(&self, style: &Self::Style) -> Sides<Sides<f32>> {
        match style {
            Border::LeftRound(_) | Border::BotRound(_) => {
                Sides::from(2.).into()
            }
            _ => Sides::from(0.).into(),
        }
    }
}

/// The types of styles of buttons
#[derive(Default, PartialEq)]
pub enum SvgButton {
    /// Default button style
    #[default]
    Default,
    /// Circle with white background
    WhiteCircle(f32),
    /// Circle with transparent background
    TransparentCircle(f32),
    RedHover,
}

impl svg_button::StyleSheet for Theme {
    type Style = SvgButton;

    fn active(&self, style: &Self::Style) -> svg_button::Appearance {
        let default = svg_button::Appearance {
            background: TRANSPARENT_BG,
            border_radius: RADIUS.into(),
            border_thickness: 0.,
            border_color: TRANSPARENT,
            svg_color: None,
        };

        match style {
            SvgButton::Default => default,
            SvgButton::WhiteCircle(r) => svg_button::Appearance {
                background: FOREGROUND_BG,
                border_radius: (*r).into(),
                svg_color: Some(BG_DARK),
                ..default
            },
            SvgButton::TransparentCircle(r) => svg_button::Appearance {
                border_radius: (*r).into(),
                ..default
            },
            _ => default,
        }
    }

    fn hovered(&self, style: &Self::Style) -> svg_button::Appearance {
        let base = svg_button::Appearance {
            ..self.active(style)
        };
        match style {
            SvgButton::WhiteCircle(_) => svg_button::Appearance {
                background: CONTRAST_BG,
                ..base
            },
            SvgButton::TransparentCircle(_) => svg_button::Appearance {
                svg_color: Some(CONTRAST),
                ..self.active(style)
            },
            SvgButton::RedHover => svg_button::Appearance {
                svg_color: Some(RED),
                ..base
            },
            _ => base,
        }
    }

    fn pressed(&self, style: &Self::Style) -> svg_button::Appearance {
        let base = svg_button::Appearance {
            ..self.hovered(style)
        };

        match style {
            SvgButton::WhiteCircle(_) => svg_button::Appearance {
                background: BRIGHT_CONTRAST_BG,
                ..base
            },
            SvgButton::TransparentCircle(_) => svg_button::Appearance {
                svg_color: Some(BRIGHT_CONTRAST),
                ..base
            },
            SvgButton::RedHover => svg_button::Appearance {
                svg_color: Some(BRIGHT_RED),
                ..base
            },
            _ => base,
        }
    }

    // fn disabled(&self, style: &Self::Style) -> button::Appearance;
}

impl line_text::StyleSheet for Theme {
    type Style = Text;

    fn foreground(&self, style: &Self::Style) -> Option<Color> {
        match style {
            Text::Default => Some(FOREGROUND),
            Text::NoForeground => None,
            Text::Contrast => Some(CONTRAST),
            Text::Gray => Some(GRAY_FG),
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub enum CursorGrad {
    #[default]
    Short,
    Long,
}

impl cursor_grad::StyleSheet for Theme {
    type Style = CursorGrad;

    fn active(&self, _style: &Self::Style) -> Option<cursor_grad::Appearance> {
        None
    }

    fn hovered(&self, style: &Self::Style) -> Option<cursor_grad::Appearance> {
        Some(cursor_grad::Appearance {
            border_radius: 6.into(),
            mouse_color: Color::from_rgba8(0x99, 0x99, 0x99, 0.05),
            fade_color: Color::from_rgba8(0x99, 0x99, 0x99, 0.),
            fade_len: match style {
                CursorGrad::Long => 700.,
                CursorGrad::Short => 100.,
            },
        })
    }
}

impl switch::StyleSheet for Theme {
    type Style = ();

    fn active(&self, style: &Self::Style) -> switch::Appearance {
        switch::Appearance {
            rail_color: DARK_CONTRAST_BG,
            ..self.inactive(style)
        }
    }

    fn inactive(&self, _style: &Self::Style) -> switch::Appearance {
        switch::Appearance {
            rail_size: Size::new(40., 20.),
            thumb_size: Size::new(16., 16.),
            rail_border_color: TRANSPARENT,
            thumb_border_color: TRANSPARENT,
            rail_border_radius: 15.into(),
            thumb_border_radius: 13.into(),
            rail_border_thickness: 0.,
            thumb_border_thickness: 0.,
            rail_color: OUTLINE_BG,
            thumb_color: FOREGROUND_BG,
            text_color: Some(FOREGROUND),
        }
    }

    fn active_hovered(&self, style: &Self::Style) -> switch::Appearance {
        switch::Appearance {
            thumb_color: CONTRAST_BG,
            text_color: Some(CONTRAST),
            ..self.active(style)
        }
    }

    fn inactive_hovered(&self, style: &Self::Style) -> switch::Appearance {
        switch::Appearance {
            thumb_color: CONTRAST_BG,
            text_color: Some(CONTRAST),
            ..self.inactive(style)
        }
    }
}
