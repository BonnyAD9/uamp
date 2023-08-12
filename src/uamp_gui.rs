use std::sync::Arc;

use iced_core::Length::{Fill, FillPortion};

use crate::{
    button, col,
    fancy_widgets::icons,
    library::{Filter, SongId},
    row, text,
    theme::{Button, Text},
    uamp_app::{ControlMsg, UampApp, UampMessage as Msg},
    wid::{
        self, button, center, center_y, container, nothing, slider, space,
        svg, wrap_box, Command, Element,
    },
};

#[derive(Clone, Copy, Debug)]
pub enum Message {
    SetPage(MainPage),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MainPage {
    Songs,
    Playlist,
}

pub struct GuiState {
    page: MainPage,
}

impl UampApp {
    pub fn gui_event(&mut self, message: Message) -> Command {
        match message {
            Message::SetPage(page) => self.gui.page = page,
        }
        Command::none()
    }

    pub fn gui(&self) -> Element {
        col![
            self.menu(),
            self.main_page(),
            self.play_menu(),
            //event_capture(|e, m, c| self.events(e, m, c))
        ]
        .into()
    }

    // menu

    fn menu(&self) -> Element {
        let make_button =
            |text: &'static str, page: MainPage| -> wid::Button {
                button(wid::text(text).style(if self.gui.page == page {
                    Text::Contrast
                } else {
                    Text::Default
                }))
                .on_press(Msg::Gui(Message::SetPage(page)))
                .height(30)
            };

        row![
            make_button("Songs", MainPage::Songs),
            make_button("Playlist", MainPage::Playlist),
        ]
        .into()
    }

    // main page

    fn main_page(&self) -> Element {
        match self.gui.page {
            MainPage::Songs => {
                self.song_list(self.library.filter(Filter::All).collect())
            }
            MainPage::Playlist => self.playlist(),
        }
    }

    // playlist

    fn playlist(&self) -> Element {
        col![
            button!("Shuffle").on_press(Msg::Control(ControlMsg::Shuffle)),
            self.song_list(self.player.playlist().as_arc())
        ]
        .height(Fill)
        .into()
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
        let button_style = if song % 2 == 0 {
            Button::ItemEven
        } else {
            Button::ItemOdd
        };

        let text_style = if Some(songs[song]) == self.player.now_playing() {
            Text::Contrast
        } else {
            Text::Default
        };

        let s = &self.library[songs[song]];

        button(text!("{} - {}", s.artist(), s.title()).style(text_style))
            .on_press(Msg::PlaySong(song, songs))
            .height(Fill)
            .width(Fill)
            .style(button_style)
            .into()
    }

    // play menu

    fn play_menu(&self) -> Element {
        row![
            button(svg(icons::PREVIOUS))
                .height(30)
                .width(30)
                .on_press(Msg::Control(ControlMsg::PrevSong)),
            self.play_pause_button(),
            button(svg(icons::NEXT))
                .height(30)
                .width(30)
                .on_press(Msg::Control(ControlMsg::NextSong)),
            self.current_song(),
            self.volume(),
        ]
        .height(30)
        .into()
    }

    fn play_pause_button(&self) -> Element {
        let icon = if self.player.is_playing() {
            icons::PAUSE
        } else {
            icons::PLAY
        };

        button(svg(icon))
            .height(30)
            .width(30)
            .on_press(Msg::Control(ControlMsg::PlayPause))
            .into()
    }

    fn current_song(&self) -> Element {
        button(if let Some(s) = self.player.now_playing() {
            let s = &self.library[s];
            text!("{} - {}", s.artist(), s.title()).width(Fill).into()
        } else {
            <iced::widget::Space as Into<Element>>::into(nothing())
        })
        .height(30)
        .width(Fill)
        .on_press(Msg::Gui(Message::SetPage(MainPage::Playlist)))
        .into()
    }

    fn volume(&self) -> Element {
        row![
            self.mute_button(),
            center(text!("{:.0}", self.player.volume() * 100.)).width(30),
            center_y(
                slider(0.0..=1., self.player.volume(), |v| {
                    Msg::Control(ControlMsg::SetVolume(v))
                })
                .step(0.001),
            )
            .width(150)
            .padding([0, 5])
        ]
        .into()
    }

    fn mute_button(&self) -> Element {
        button(svg(if self.player.mute() {
            icons::NO_VOLUME
        } else {
            icons::VOLUME
        }))
        .width(30)
        .height(30)
        .on_press(Msg::Control(ControlMsg::ToggleMute))
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
