use std::{default, f32::consts::PI, sync::Condvar};

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
    Background, Color, Degrees, Gradient, Padding, Vector,
};
use serde::de;

use super::widgets::{border, line_text, sides::Sides, svg_button, wrap_box, cursor_grad};

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

/// The primary color
const PRIMARY: Color = const_color!(0x222222);
/// Primary color as background
const PRIMARY_BG: Background = Background::Color(PRIMARY);
/// Secondary color
const SECONDARY: Color = const_color!(0x181818);
/// Secondary color as background
const SECONDARY_BG: Background = Background::Color(SECONDARY);
/// The outline color
const OUTLINE: Color = const_color!(0x555555);
/// The outline color as background
const OUTLINE_BG: Background = Background::Color(OUTLINE);
/// The foreground color
const FOREGROUND: Color = const_color!(0xEEEEEE);
/// The inactice foreground color
const DARK_FOREGROUND: Color = const_color!(0x777777);
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
const DARK_CONTRAST: Color = const_color!(0xBBBB33);
/// The contras color as background
const DARK_CONTRAST_BG: Background = Background::Color(DARK_CONTRAST);
/// Brighter version of contrast color
const BRIGHT_CONTRAST: Color = const_color!(0xEEEE33);
/// The contras color as background
const BRIGHT_CONTRAST_BG: Background = Background::Color(BRIGHT_CONTRAST);
/// The border radius
const RADIUS: f32 = 4.;
/// The border thickness
const THICKNESS: f32 = 1.;

/// The theme of uamp app
#[derive(Default, Clone)]
pub struct Theme {}

impl application::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> application::Appearance {
        application::Appearance {
            background_color: PRIMARY,
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
    /// Circle with white background
    WhiteCircle(f32),
    /// Circle with transparent background
    TransparentCircle(f32),
    /// Gradient that is lighter on top and bottom
    GradItem,
    /// Odd items in list
    ItemOdd,
    /// Even items in list
    ItemEven,
    Item,
    MenuItem,
}

impl button::StyleSheet for Theme {
    type Style = Button;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let default = button::Appearance {
            shadow_offset: Vector::ZERO,
            background: Some(SECONDARY_BG),
            border_radius: RADIUS.into(),
            border_width: THICKNESS,
            border_color: OUTLINE,
            text_color: FOREGROUND,
        };

        match style {
            Button::Default => default,
            Button::ItemEven => button::Appearance {
                border_color: PRESSED,
                ..default
            },
            Button::ItemOdd => button::Appearance {
                background: Some(PRIMARY_BG),
                border_width: 0.,
                ..default
            },
            Button::WhiteCircle(r) => button::Appearance {
                background: Some(Background::Color(FOREGROUND)),
                border_width: 0.,
                border_radius: (*r).into(),
                ..default
            },
            Button::TransparentCircle(r) => button::Appearance {
                background: Some(Background::Color(Color::TRANSPARENT)),
                border_width: 0.,
                border_radius: (*r).into(),
                ..default
            },
            Button::GradItem => button::Appearance {
                background: Some(Background::Color(Color::TRANSPARENT)),
                border_width: 0.,
                border_radius: 4.0.into(),
                ..default
            },
            Button::Item => button::Appearance {
                background: None,
                border_width: 0.,
                border_radius: 6.0.into(),
                ..default
            },
            Button::MenuItem => button::Appearance {
                background: None,
                border_width: 0.,
                border_radius: 6.0.into(),
                text_color: FOREGROUND,
                ..default
            }
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let base = button::Appearance {
            background: Some(PRESSED_BG),
            border_color: CONTRAST,
            ..self.active(style)
        };
        match style {
            Button::ItemOdd => button::Appearance {
                border_width: THICKNESS,
                border_color: CONTRAST,
                ..base
            },
            Button::WhiteCircle(_) => button::Appearance {
                background: Some(CONTRAST_BG),
                ..base
            },
            Button::TransparentCircle(_) => button::Appearance {
                background: Some(Background::Color(Color::TRANSPARENT)),
                ..base
            },
            Button::Item => button::Appearance {
                background: Some(Background::Color(Color::TRANSPARENT)),
                ..base
            },
            Button::MenuItem => button::Appearance {
                background: Some(Background::Color(Color::TRANSPARENT)),
                text_color: CONTRAST,
                ..base
            },
            _ => base,
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        let base = button::Appearance {
            background: Some(SELECTED_BG),
            border_color: BRIGHT_CONTRAST,
            ..self.active(style)
        };

        match style {
            Button::ItemOdd => button::Appearance {
                border_width: THICKNESS,
                ..base
            },
            Button::WhiteCircle(_) => button::Appearance {
                background: Some(BRIGHT_CONTRAST_BG),
                ..base
            },
            Button::TransparentCircle(_) => button::Appearance {
                background: Some(Background::Color(Color::TRANSPARENT)),
                ..base
            },
            Button::Item => button::Appearance {
                background: Some(Background::Color(const_color!(0x222222))),
                ..base
            },
            Button::MenuItem => button::Appearance {
                background: Some(Background::Color(const_color!(0x222222))),
                text_color: BRIGHT_CONTRAST,
                ..base
            },
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
            background: SECONDARY_BG,
            icon_color: CONTRAST,
            border_radius: RADIUS.into(),
            border_width: THICKNESS,
            border_color: OUTLINE,
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
    Black,
    ToInvis,
    FromInvis,
    TopGrad,
}

impl container::StyleSheet for Theme {
    type Style = Container;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let default = container::Appearance {
            ..container::Appearance::default()
        };

        match style {
            Container::Default => default,
            Container::Gray => container::Appearance {
                background: Some(Background::Color(const_color!(0x1E1E1E)),),
                ..default
            },
            Container::Black => container::Appearance {
                background: Some(Background::Color(Color::BLACK)),
                ..default
            },
            Container::ToInvis => container::Appearance {
                background: Some(Background::Gradient(Gradient::Linear(Linear::new(Degrees(180.)).add_stops([
                    ColorStop { offset: 0., color: const_color!(0x1E1E1E) },
                    ColorStop { offset: 1., color: Color::from_rgba8(0x1E, 0x1E, 0x1E, 0.) },
                ])))),
                ..default
            },
            Container::FromInvis => container::Appearance {
                background: Some(Background::Gradient(Gradient::Linear(Linear::new(Degrees(180.)).add_stops([
                    ColorStop { offset: 0., color: Color::from_rgba8(0x1E, 0x1E, 0x1E, 0.) },
                    ColorStop { offset: 1., color: const_color!(0x1E1E1E) },
                ])))),
                ..default
            },
            Container::TopGrad => container::Appearance {
                background: Some(Background::Gradient(Gradient::Linear(
                    Linear::new(Degrees(270.)).add_stops([
                        ColorStop {
                            offset: 0.,
                            color: const_color!(0x1E1E1E),
                        },
                        ColorStop {
                            offset: 0.8,
                            color: const_color!(0x181818),
                        },
                    ]),
                ))),
                ..default
            }
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
                color: Color::TRANSPARENT,
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
            background: PRIMARY_BG,
            border_width: THICKNESS,
            border_radius: RADIUS.into(),
            border_color: OUTLINE,
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
            placeholder_color: DARK_FOREGROUND,
            handle_color: CONTRAST,
            background: SECONDARY_BG,
            border_radius: RADIUS.into(),
            border_width: THICKNESS,
            border_color: OUTLINE,
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
            background: SECONDARY_BG,
            dot_color: if is_selected { CONTRAST } else { SELECTED },
            border_width: THICKNESS,
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
            background: PRIMARY,
            background_border: None,
            foreground: if is_active { DARK_FOREGROUND } else { SELECTED },
            foreground_border: None,
        }
    }

    fn hovered(
        &self,
        style: &Self::Style,
        is_active: bool,
    ) -> toggler::Appearance {
        toggler::Appearance {
            background: SELECTED,
            ..self.active(style, is_active)
        }
    }
}

impl pane_grid::StyleSheet for Theme {
    type Style = ();

    fn picked_split(&self, _style: &Self::Style) -> Option<pane_grid::Line> {
        Some(pane_grid::Line {
            color: OUTLINE,
            width: THICKNESS,
        })
    }

    fn hovered_split(&self, _style: &Self::Style) -> Option<pane_grid::Line> {
        Some(pane_grid::Line {
            color: CONTRAST,
            width: THICKNESS,
        })
    }

    fn hovered_region(&self, _style: &Self::Style) -> pane_grid::Appearance {
        pane_grid::Appearance {
            background: SELECTED_BG,
            border_width: THICKNESS,
            border_color: OUTLINE,
            border_radius: RADIUS.into(),
        }
    }
}

impl progress_bar::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> progress_bar::Appearance {
        progress_bar::Appearance {
            background: SECONDARY_BG,
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
            width: THICKNESS as u16,
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
    /// Use white
    Light,
    Gray,
    /// Use black
    Dark,
}

impl svg::StyleSheet for Theme {
    type Style = Svg;

    fn appearance(&self, style: &Self::Style) -> svg::Appearance {
        match style {
            Svg::Original => svg::Appearance::default(),
            Svg::Light => svg::Appearance {
                color: Some(FOREGROUND),
            },
            Svg::Dark => svg::Appearance {
                color: Some(const_color!(0x181818)),
            },
            Svg::Gray => svg::Appearance {
                color: Some(const_color!(0x777777)),
            },
        }
    }
}

impl scrollable::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> scrollable::Scrollbar {
        scrollable::Scrollbar {
            background: None,
            border_radius: RADIUS.into(),
            border_width: THICKNESS,
            border_color: OUTLINE,
            scroller: scrollable::Scroller {
                color: PRIMARY,
                border_radius: RADIUS.into(),
                border_width: THICKNESS,
                border_color: OUTLINE,
            },
        }
    }

    fn hovered(
        &self,
        style: &Self::Style,
        is_mouse_over_scrollbar: bool,
    ) -> scrollable::Scrollbar {
        let base = self.active(style);
        scrollable::Scrollbar {
            scroller: scrollable::Scroller {
                color: if is_mouse_over_scrollbar {
                    SELECTED
                } else {
                    PRIMARY
                },
                ..base.scroller
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
    /// Text with contrast color as foreground
    Contrast,
    Gray,
    Dark,
}

impl text::StyleSheet for Theme {
    type Style = Text;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
        text::Appearance {
            color: match style {
                Text::Default => Some(FOREGROUND),
                Text::NoForeground => None,
                Text::Contrast => Some(CONTRAST),
                Text::Gray => Some(const_color!(0x777777)),
                Text::Dark => Some(const_color!(0x141414)),
            },
        }
    }
}

impl text_input::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: SECONDARY_BG,
            border_radius: RADIUS.into(),
            border_width: THICKNESS,
            border_color: OUTLINE,
            icon_color: PRIMARY,
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        let base = self.active(style);
        text_input::Appearance {
            border_color: PRIMARY,
            ..base
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        DARK_FOREGROUND
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        FOREGROUND
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        FOREGROUND
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        FOREGROUND
    }

    fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
        self.active(style)
    }

    // fn disabled_color(&self, style: &Self::Style) -> Color;
}

impl wrap_box::StyleSheet for Theme {
    type Style = ();

    fn background(
        &self,
        _style: &Self::Style,
        _pos: wrap_box::MousePos,
    ) -> wrap_box::SquareStyle {
        wrap_box::SquareStyle {
            background: Background::Color(const_color!(0x181818)),
            border: Color::BLACK,
            border_thickness: 0.,
            border_radius: 0.0.into(),
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
            background: Background::Color(Color::TRANSPARENT),
            border: Color::TRANSPARENT,
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
                foreground: DARK_FOREGROUND,
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

            wrap_box::ButtonStyle {
                square,
                foreground,
            }
        }
    }

    fn thumb_style(
        &self,
        _style: &Self::Style,
        pos: wrap_box::MousePos,
        pressed: bool,
        _relative_scroll: f32,
    ) -> wrap_box::SquareStyle {
        let square = wrap_box::SquareStyle {
            background: PRIMARY_BG,
            border: OUTLINE,
            border_thickness: 0.,
            border_radius: 6.0.into(),
        };

        if pressed {
            wrap_box::SquareStyle {
                background: SELECTED_BG,
                border: BRIGHT_CONTRAST,
                ..square
            }
        } else if pos == wrap_box::MousePos::DirectlyOver {
            wrap_box::SquareStyle {
                background: PRESSED_BG,
                border: CONTRAST,
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
            background: SECONDARY_BG,
            border: PRESSED,
            border_thickness: 0.0,
            border_radius: if is_start {
                [6., 6., 0., 0.].into()
            } else {
                [0., 0., 6., 6.].into()
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
    TopGrad,
    SongItem,
    Bot,
    BotRound(bool),
    LeftRound(bool),
}

impl border::StyleSheet for Theme {
    type Style = Border;

    fn background(&self, style: &Self::Style) -> Background {
        match style {
            Border::None => Background::Color(Color::TRANSPARENT),
            Border::TopGrad=> Background::Gradient(Gradient::Linear(
                Linear::new(Degrees(270.)).add_stops([
                    ColorStop {
                        offset: 0.,
                        color: const_color!(0x1E1E1E),
                    },
                    ColorStop {
                        offset: 0.8,
                        color: const_color!(0x181818),
                    },
                ]),
            )),
            _ => Background::Color(Color::TRANSPARENT),
        }
    }

    fn border_thickness(&self, style: &Self::Style) -> Sides<f32> {
        match style {
            Border::None => 0.into(),
            Border::TopGrad => 0.into(),
            Border::Bot => [0, 0, 2, 0].into(),
            Border::BotRound(true) => [0, 0, 4, 0].into(),
            Border::BotRound(false) => 0.into(),
            Border::LeftRound(true) => [0, 0, 0, 4].into(),
            Border::LeftRound(false) => 0.into(),
            Border::SongItem => [1, 0, 0, 0].into(),
        }
    }

    fn border_radius(&self, style: &Self::Style) -> Sides<f32> {
        match style {
            Border::None => 0.into(),
            Border::TopGrad => 0.into(),
            Border::Bot => 15.into(),
            Border::BotRound(_) => 0.into(),
            Border::LeftRound(_) => 0.into(),
            Border::SongItem => 6.into(),
        }
    }

    fn border_color(&self, style: &Self::Style) -> Sides<Background> {
        match style {
            Border::None => OUTLINE_BG.into(),
            Border::TopGrad => OUTLINE_BG.into(),
            Border::Bot => Background::Color(const_color!(0x333333)).into(),
            Border::BotRound(_) => CONTRAST_BG.into(),
            Border::LeftRound(_) => CONTRAST_BG.into(),
            Border::SongItem => Background::Color(const_color!(0x444444)).into(),
        }
    }

    fn corner_color(&self, style: &Self::Style) -> Sides<Color> {
        OUTLINE.into()
    }

    fn border_border_radius(&self, style: &Self::Style) -> Sides<Sides<f32>> {
        match style {
            Border::LeftRound(_) => Sides::from(2.).into(),
            _ => Sides::from(0.).into()
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
    /// Odd items in list
    ItemOdd,
    /// Even items in list
    ItemEven,
}

impl svg_button::StyleSheet for Theme {
    type Style = SvgButton;

    fn active(&self, style: &Self::Style) -> svg_button::Appearance {
        let default = svg_button::Appearance {
            background: SECONDARY_BG,
            border_radius: RADIUS.into(),
            border_thickness: THICKNESS,
            border_color: OUTLINE,
            svg_color: None,
        };

        match style {
            SvgButton::Default => default,
            SvgButton::ItemEven => svg_button::Appearance {
                border_color: PRESSED,
                ..default
            },
            SvgButton::ItemOdd => svg_button::Appearance {
                background: PRIMARY_BG,
                border_thickness: 0.,
                ..default
            },
            SvgButton::WhiteCircle(r) => svg_button::Appearance {
                background: Background::Color(FOREGROUND),
                border_thickness: 0.,
                border_radius: (*r).into(),
                ..default
            },
            SvgButton::TransparentCircle(r) => svg_button::Appearance {
                background: Background::Color(Color::TRANSPARENT),
                border_thickness: 0.,
                border_radius: (*r).into(),
                svg_color: Some(FOREGROUND),
                ..default
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> svg_button::Appearance {
        let base = svg_button::Appearance {
            background: PRESSED_BG,
            border_color: CONTRAST,
            ..self.active(style)
        };
        match style {
            SvgButton::ItemOdd => svg_button::Appearance {
                border_thickness: THICKNESS,
                border_color: CONTRAST,
                ..base
            },
            SvgButton::WhiteCircle(_) => svg_button::Appearance {
                background: CONTRAST_BG,
                ..base
            },
            SvgButton::TransparentCircle(_) => svg_button::Appearance {
                svg_color: Some(CONTRAST),
                background: Background::Color(Color::TRANSPARENT),
                ..self.active(style)
            },
            _ => base,
        }
    }

    fn pressed(&self, style: &Self::Style) -> svg_button::Appearance {
        let base = svg_button::Appearance {
            background: SELECTED_BG,
            border_color: BRIGHT_CONTRAST,
            ..self.active(style)
        };

        match style {
            SvgButton::ItemOdd => svg_button::Appearance {
                border_thickness: THICKNESS,
                ..base
            },
            SvgButton::WhiteCircle(_) => svg_button::Appearance {
                background: BRIGHT_CONTRAST_BG,
                ..base
            },
            SvgButton::TransparentCircle(_) => svg_button::Appearance {
                svg_color: Some(BRIGHT_CONTRAST),
                background: Background::Color(Color::TRANSPARENT),
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
            Text::Gray => Some(const_color!(0x777777)),
            Text::Dark => Some(const_color!(0x101010)),
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
            }
        })
    }
}
