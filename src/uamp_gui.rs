use std::sync::Arc;

use iced_core::Length::Fill;

use crate::{
    button, col,
    fancy_widgets::icons,
    library::{Filter, SongId},
    text,
    theme::Button,
    uamp_app::{UampApp, UampMessage as Msg},
    wid::{button, svg, wrap_box, Command, Element},
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
            MainPage::Songs => {
                self.song_list(self.library.filter(Filter::All).collect())
            }
        }
    }

    // song list

    fn song_list(&self, songs: Arc<[SongId]>) -> Element {
        wrap_box(
            (0..songs.len())
                .map(|i| self.song_list_item(i, songs.clone()))
                .collect(),
        )
        .item_height(32)
        .from_layout_style(&self.theme)
        .into()
    }

    fn song_list_item(
        &self,
        song: usize,
        songs: Arc<[SongId]>,
    ) -> Element<'static> {
        let style = if song % 2 == 0 {
            Button::ItemEven
        } else {
            Button::ItemOdd
        };

        let s = &self.library[songs[song]];

        button!("{} - {}", s.artist(), s.title())
            .on_press(Msg::PlaySong(song, songs))
            .height(Fill)
            .width(Fill)
            .style(style)
            .into()
    }

    // play menu

    fn play_menu(&self) -> Element {
        let icon = if self.player.is_playing() {
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
