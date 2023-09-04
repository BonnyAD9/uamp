use iced::{
    application,
    overlay::menu,
    widget::{
        button, checkbox, container, pane_grid, pick_list, progress_bar,
        radio, rule, scrollable, slider, svg, text, text_input, toggler,
    },
};
use iced_core::{Background, Color, Padding, Vector};

use super::widgets::wrap_box;

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
const CONTRAST: Color = const_color!(0xCCCC00);
/// The contras color as background
const CONTRAST_BG: Background = Background::Color(CONTRAST);
/// Brighter version of contrast color
const BRIGHT_CONTRAST: Color = const_color!(0xEEEE00);
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
    /// Odd items in list
    ItemOdd,
    /// Even items in list
    ItemEven,
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

impl container::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance::default()
    }
}

impl slider::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            rail: slider::Rail {
                colors: (CONTRAST, OUTLINE),
                width: THICKNESS * 2.,
                border_radius: RADIUS.into(),
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 5. },
                color: SECONDARY,
                border_width: THICKNESS,
                border_color: OUTLINE,
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> slider::Appearance {
        let base = self.active(style);
        slider::Appearance {
            handle: slider::Handle {
                color: SELECTED,
                border_color: CONTRAST,
                ..base.handle
            },
            ..base
        }
    }

    fn dragging(&self, style: &Self::Style) -> slider::Appearance {
        let base = self.active(style);
        slider::Appearance {
            handle: slider::Handle {
                color: PRESSED,
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

impl svg::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> svg::Appearance {
        svg::Appearance::default()
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
#[derive(Clone, Default)]
pub enum Text {
    /// The default text style
    #[default]
    Default,
    /// Text with contrast color as foreground
    Contrast,
}

impl text::StyleSheet for Theme {
    type Style = Text;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
        text::Appearance {
            color: match style {
                Text::Default => Some(FOREGROUND),
                Text::Contrast => Some(CONTRAST),
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
            background: PRIMARY_BG,
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
            background: PRIMARY_BG,
            border: OUTLINE,
            border_thickness: THICKNESS,
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

            let square = if pressed {
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
                wrap_box::SquareStyle {
                    border_thickness: 0.,
                    ..square
                }
            };

            wrap_box::ButtonStyle {
                square,
                foreground: FOREGROUND,
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
            border_thickness: THICKNESS,
            border_radius: RADIUS.into(),
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
            border_thickness: THICKNESS,
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
            padding: Some(Padding {
                left: 30.,
                right: 30.,
                top: 0.,
                bottom: 0.,
            }),
            spacing: (None, Some(1.)),
            ..wrap_box::LayoutStyle::default()
        }
    }
}
