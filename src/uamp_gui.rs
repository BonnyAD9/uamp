use std::sync::Arc;

use iced_native::Length::Fill;

use crate::{
    button, col,
    fancy_widgets::icons,
    library::Filter,
    text,
    theme::Button,
    uamp_app::{UampApp, UampMessage as Msg},
    wid::{button, svg, wrap_box, Command, Element, IteratorFn},
};

#[derive(Clone, Copy, Debug)]
pub enum Message {}

enum MainPage {
    Songs,
}

pub struct GuiState {
    page: MainPage,
}

impl UampApp {
    pub fn gui_event(&mut self, _message: Message) -> Command {
        Command::none()
    }

    pub fn gui(&self) -> Element {
        col![
            self.main_page(),
            self.play_menu(),
            //event_capture(|e, m, c| self.events(e, m, c))
        ]
        .into()
    }

    pub fn main_page(&self) -> Element {
        match self.gui.page {
            MainPage::Songs => self.song_list(|l| l.filter(Filter::All)),
        }
    }

    // song list

    fn song_list(&self, songs: impl IteratorFn + 'static) -> Element {
        let mut i = 0;

        let songs = Arc::new(songs);

        wrap_box(
            songs(&self.library)
                .map(|s| {
                    i += 1;
                    self.song_list_item(s, s % 2 == 0, songs.clone())
                })
                .collect(),
        )
        .item_height(30)
        .from_layout_style(&self.theme)
        .into()
    }

    fn song_list_item(
        &self,
        song: usize,
        even: bool,
        songs: Arc<dyn IteratorFn>,
    ) -> Element<'static> {
        let style = if even {
            Button::ItemEven
        } else {
            Button::ItemOdd
        };

        let s = &self.library[song];

        button!("{} - {}", s.artist(), s.title())
            .on_press(Msg::PlaySong(song, songs))
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

        button(svg(icon))
            .height(30)
            .width(30)
            .on_press(Msg::PlayPause)
            .into()
    }
}

impl GuiState {
    pub fn new() -> Self {
        GuiState {
            page: MainPage::Songs,
        }
    }
}

impl Default for GuiState {
    fn default() -> Self {
        GuiState::new()
    }
}
