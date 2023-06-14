use iced_native::Length::Fill;

use crate::{
    button, col,
    fancy_widgets::icons,
    library::Filter,
    text,
    theme::Button,
    uamp_app::{UampApp, UampMessage as Msg},
    wid::{button, svg, wrap_box, Element},
};

impl UampApp {
    pub fn gui(&self) -> Element {
        col![
            self.song_list(self.library.filter(Filter::All)),
            self.play_menu(),
        ]
        .into()
    }

    // song list

    fn song_list(&self, songs: impl Iterator<Item = usize>) -> Element {
        let mut i = 0;

        wrap_box(
            songs
                .map(|s| {
                    i += 1;
                    self.song_list_item(s, s % 2 == 0)
                })
                .collect(),
        )
        .item_height(30)
        .from_layout_style(&self.theme)
        .into()
    }

    fn song_list_item(&self, song: usize, even: bool) -> Element<'static> {
        let style = if even {
            Button::ItemEven
        } else {
            Button::ItemOdd
        };

        let s = &self.library[song];

        button!("{} - {}", s.artist(), s.title())
            .on_press(Msg::PlaySong(song))
            .height(Fill)
            .width(Fill)
            .style(style)
            .into()
    }

    // play menu

    fn play_menu(&self) -> Element {
        let icon = if self.now_playing.is_playing() {
            icons::PAUSE
        } else {
            icons::PLAY
        };

        button(svg(icon)).height(30).width(30).on_press(Msg::PlayPause).into()
    }
}
