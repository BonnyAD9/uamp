use iced::application;
use iced::color;

#[derive(Default)]
pub struct Theme {}

impl application::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> application::Appearance {
        application::Appearance {
            background_color: color!(0x181818),
            text_color: color!(0xEEEEEE),
        }
    }
}
