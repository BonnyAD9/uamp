use iced_native::Length::Fill;

use crate::{
    col,
    fancy_widgets::icons,
    theme::Button,
    uamp_app::{UampApp, UampMessage as Msg},
    wid::{button, svg, text, wrap_box, Element},
};

impl UampApp {
    pub fn gui(&self) -> Element {
        _ = self.config;
        let mut c = 0;
        let list = wrap_box(
            self.library
                .iter()
                .map(|s| {
                    c += 1;
                    button(text(format!("{} - {}", s.artist(), s.title())))
                        .style(if c % 2 == 0 {
                            Button::ItemEven
                        } else {
                            Button::ItemOdd
                        })
                        .on_press(Msg::PlaySong(c - 1))
                        .width(Fill)
                        .height(Fill)
                        .into()
                })
                .collect(),
        )
        .item_height(30)
        .from_layout_style(&self.theme);

        let now_playing = button(svg(if self.now_playing.is_playing() {
            icons::PAUSE
        } else {
            icons::PLAY
        }))
        .on_press(Msg::PlayPause)
        .width(30.)
        .height(30.);

        col![list.height(Fill), now_playing,].into()
    }
}
