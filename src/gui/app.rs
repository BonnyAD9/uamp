use std::{sync::Arc, time::Duration};

use iced_core::Length::{Fill, Shrink};

use crate::{
    app::UampApp,
    button, col,
    core::msg::{ComMsg, ControlMsg, Msg},
    library::{Filter, SongId},
    player::TimeStamp,
    row, text,
};

use super::{
    ids::*,
    msg::Message,
    theme::{Button, Text},
    wid::{
        self, button, center, center_y, nothing, slider, space, svg, wrap_box,
        Command, Element, WrapBoxState,
    },
    widgets::icons,
};

/// The state of the gui
pub struct GuiState {
    /// The current page
    page: MainPage,
    /// States of WrapBoxes
    wb_states: Vec<WrapBoxState>,

    song_timestamp: Option<TimeStamp>,
    seek_drag: Option<Duration>,
}

/// Available main menu pages
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MainPage {
    /// All songs page
    Songs,
    /// Current playlist page
    Playlist,
}

impl GuiState {
    /// Creates default gui state
    pub fn new() -> Self {
        GuiState {
            page: MainPage::Songs,
            wb_states: vec![WrapBoxState::default(); WB_STATE_COUNT],
            song_timestamp: None,
            seek_drag: None,
        }
    }
}

impl Default for GuiState {
    fn default() -> Self {
        GuiState::new()
    }
}

impl UampApp {
    /// handles gui events
    pub fn gui_event(&mut self, message: Message) -> ComMsg {
        match message {
            Message::SetPage(page) => self.gui.page = page,
            Message::SeekSliderMove(d) => self.gui.seek_drag = Some(d),
            Message::SeekSliderEnd => {
                return ComMsg::Msg(Msg::Control(ControlMsg::SeekTo(
                    self.gui.seek_drag.take().unwrap_or_default(),
                )))
            }
            Message::Tick => {
                self.gui.song_timestamp = self.player.timestamp();
            }
        }
        ComMsg::Command(Command::none())
    }

    /// Generates the gui
    pub fn gui(&self) -> Element {
        col![
            self.menu(),
            self.main_page(),
            self.play_menu(),
            //event_capture(|e, m, c| self.events(e, m, c))
        ]
        .into()
    }
}

impl UampApp {
    /// Creates the menu
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
            space(Fill, Shrink),
            button!("Find Songs")
                .on_press(Msg::Control(ControlMsg::LoadNewSongs))
        ]
        .into()
    }

    // main page

    /// Creates the main page
    fn main_page(&self) -> Element {
        match self.gui.page {
            MainPage::Songs => self.song_list(
                self.library.filter(Filter::All).collect(),
                &self.gui.wb_states[WB_SONGS],
            ),
            MainPage::Playlist => self.playlist(),
        }
    }

    // playlist

    /// Creates the playlist page
    fn playlist(&self) -> Element {
        col![
            button!("Shuffle").on_press(Msg::Control(ControlMsg::Shuffle)),
            self.song_list(
                self.player.playlist().as_arc(),
                &self.gui.wb_states[WB_PLAYLIST]
            )
        ]
        .height(Fill)
        .into()
    }

    // song list

    /// Creates a song list
    fn song_list<'a>(
        &'a self,
        songs: Arc<[SongId]>,
        state: &'a WrapBoxState,
    ) -> Element {
        wrap_box(
            (0..songs.len())
                .map(|i| self.song_list_item(i, songs.clone()))
                .collect(),
            state,
        )
        .item_height(32)
        .from_layout_style(&self.theme)
        .into()
    }

    /// Creates a song list item
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

    /// Creates the play menu
    fn play_menu(&self) -> Element {
        col![
            row![
                button(svg(icons::REWIND))
                    .height(30)
                    .width(30)
                    .on_press(Msg::Control(ControlMsg::Rewind(None))),
                button(svg(icons::PREVIOUS))
                    .height(30)
                    .width(30)
                    .on_press(Msg::Control(ControlMsg::PrevSong(1))),
                self.play_pause_button(),
                button(svg(icons::NEXT))
                    .height(30)
                    .width(30)
                    .on_press(Msg::Control(ControlMsg::NextSong(1))),
                button(svg(icons::FAST_FORWARD))
                    .height(30)
                    .width(30)
                    .on_press(Msg::Control(ControlMsg::FastForward(None))),
                self.current_song(),
                self.volume(),
            ]
            .height(30),
            self.seek_slider(),
        ]
        .into()
    }

    /// Creates the play/pause button
    fn play_pause_button(&self) -> Element {
        let icon = if self.player.is_playing() {
            icons::PAUSE
        } else {
            icons::PLAY
        };

        button(svg(icon))
            .height(30)
            .width(30)
            .on_press(Msg::Control(ControlMsg::PlayPause(None)))
            .into()
    }

    /// Shows the current song
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

    /// Creates the volume controls
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

    /// Creates the mute button
    fn mute_button(&self) -> Element {
        button(svg(if self.player.mute() {
            icons::NO_VOLUME
        } else {
            icons::VOLUME
        }))
        .width(30)
        .height(30)
        .on_press(Msg::Control(ControlMsg::Mute(None)))
        .into()
    }

    fn seek_slider(&self) -> Element {
        let mut ts = match self.gui.song_timestamp {
            Some(ts) => ts,
            None => {
                return row![
                    center(text!("--:--")).width(50).height(30),
                    center_y(slider(0.0..=1., 0., |_| {
                        Msg::Gui(Message::SeekSliderMove(Duration::ZERO))
                    }))
                    .width(Fill)
                    .height(30),
                    center(text!("--:--")).width(50).height(30),
                ]
                .into()
            }
        };

        if let Some(d) = self.gui.seek_drag {
            ts.current = d;
        }
        row![
            center(text!(
                "{}:{:0>2}",
                ts.current.as_secs() / 60,
                ts.current.as_secs() % 60
            ))
            .width(50)
            .height(30),
            center_y(
                slider(
                    0.0..=ts.total.as_secs_f32(),
                    ts.current.as_secs_f32(),
                    |c| Msg::Gui(Message::SeekSliderMove(
                        Duration::from_secs_f32(c)
                    )),
                )
                .on_release(Msg::Gui(Message::SeekSliderEnd))
                .step(0.1)
            )
            .width(Fill)
            .height(30),
            center(text!(
                "{}:{:0>2}",
                ts.total.as_secs() / 60,
                ts.total.as_secs() % 60
            ))
            .width(50)
            .height(30),
        ]
        .into()
    }
}
