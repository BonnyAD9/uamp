use iced_core::{layout::Limits, Length, Size};

///! This module contains custom widgets for iced that are either
///! faster, more customizable, or with more features.
///!
///! I also contains module with svg icons.
pub mod border;
pub mod cursor_grad;
pub mod grid;
pub mod icons;
pub mod line_text;
pub mod sides;
pub mod svg_button;
pub mod switch;
pub mod wrap_box;

fn limit_size(lim: &Limits, width: Length, height: Length) -> Size {
    let w = match width {
        Length::Fill | Length::FillPortion(_) => lim.max().width,
        Length::Shrink => lim.min().width,
        Length::Fixed(n) => n,
    };

    let h = match height {
        Length::Fill | Length::FillPortion(_) => lim.max().height,
        Length::Shrink => lim.min().height,
        Length::Fixed(n) => n,
    };

    Size::new(w, h)
}
