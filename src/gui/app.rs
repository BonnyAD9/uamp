use std::{sync::Arc, time::Duration, default, cell::Cell, path::Path, fs::{File, create_dir_all}};

use iced_core::Length::{Fill, Shrink};
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{
    app::UampApp,
    button, col,
    core::{
        extensions::duration_to_string,
        msg::{ComMsg, ControlMsg, Msg},
        Result
    },
    library::{Filter, SongId},
    player::TimeStamp,
    row, text, gen_struct, config::Config,
};

use super::{
    ids::*,
    msg::Message,
    theme::{Button, Text},
    wid::{
        self, button, center, center_y, nothing, slider, space, svg, wrap_box,
        Command, Element, WrapBoxState,
    },
    widgets::icons, WinMessage,
};

gen_struct! {
    /// The state of the gui
    #[derive(Serialize, Deserialize)]
    pub GuiState {
        // fields passed by reference
        ; // fields passed by value
        window_x: i32 { pub, pri } => () i32::MAX,
        window_y: i32 { pub, pri } => () i32::MAX,
        window_width: u32 { pub, pri } => () u32::MAX,
        window_height: u32 { pub, pri } => () u32::MAX,
        ; // other fields
        /// The current page
        #[serde(skip)]
        page: MainPage,
        /// States of WrapBoxes
        #[serde(skip, default = "default_states")]
        wb_states: Vec<WrapBoxState>,

        #[serde(skip)]
        song_timestamp: Option<TimeStamp>,
        #[serde(skip)]
        seek_drag: Option<Duration>,
        ; // auto gen attribute
        #[serde(skip)]
    }
}

/// Available main menu pages
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum MainPage {
    /// All songs page
    #[default]
    Songs,
    /// Current playlist page
    Playlist,
}

impl GuiState {
    /// Creates default gui state
    pub fn new() -> Self {
        GuiState {
            page: MainPage::Songs,
            wb_states: default_states(),
            song_timestamp: None,
            seek_drag: None,
            window_x: default_window_x(),
            window_y: default_window_y(),
            window_width: default_window_width(),
            window_height: default_window_height(),
            change: Cell::new(false),
        }
    }

    /// Saves the gui state to the default location. It is not saved if there
    /// was no change.
    pub fn to_default_json(&self, conf: &Config) -> Result<()> {
        if !self.change.get() {
            return Ok(());
        }
        if let Some(p) = conf.gui_state_path() {
            self.to_json(p)?;
        }
        self.change.set(false);
        Ok(())
    }

    /// Loads gui state from default json. If it fails, creates default.
    pub fn from_default_json(conf: &Config) -> Self {
        if let Some(p) = conf.gui_state_path() {
            Self::from_json(p)
        } else {
            Self::default()
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
            Message::SetPage(page) => {
                self.gui.page = page;
                if page == MainPage::Playlist {
                    self.gui.wb_states[WB_PLAYLIST].get_mut().scroll_to =
                        self.player.current();
                }
            }
            Message::SeekSliderMove(d) => self.gui.seek_drag = Some(d),
            Message::SeekSliderEnd => {
                return ComMsg::Msg(Msg::Control(ControlMsg::SeekTo(
                    self.gui.seek_drag.take().unwrap_or_default(),
                )))
            }
            Message::Tick => {
                self.gui.song_timestamp = self.player.timestamp();
                let st = self.gui.wb_states[WB_PLAYLIST].get_mut();
                if st.scroll_to.is_some() {
                    st.scroll_to = self.player.current();
                }
            }
        }
        ComMsg::Command(Command::none())
    }

    pub fn win_event(&mut self, message: WinMessage) -> ComMsg {
        match message {
            WinMessage::Position(x, y) => {
                self.gui.window_x_set(x);
                self.gui.window_y_set(y);
            },
            WinMessage::Size(w, h) => {
                self.gui.window_width_set(w);
                self.gui.window_height_set(h);
            },
        }

        ComMsg::none()
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

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl GuiState {
    /// Loads gui state from the given path, if it fails creates default.
    fn from_json<P>(path: P) -> Self where P: AsRef<Path> {
        if let Ok(file) = File::open(path.as_ref()) {
            match serde_json::from_reader(file) {
                Ok(g) => g,
                Err(e) => {
                    error!("Failed to load gui state: {e}");
                    GuiState::default()
                }
            }
        } else {
            info!("Gui file {:?} doesn't exist", path.as_ref());
            Self::default()
        }
    }

    /// Saves the gui state to the given file.
    fn to_json<P>(&self, path: P) -> Result<()> where P: AsRef<Path> {
        if let Some(par) = path.as_ref().parent() {
            create_dir_all(par)?;
        }

        serde_json::to_writer(File::create(path)?, self)?;
        Ok(())
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

    /// Creates seek slider that updates with tick
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
            center(text!("{}", duration_to_string(ts.current, true)))
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
            center(text!("{}", duration_to_string(ts.total, true)))
                .width(50)
                .height(30),
        ]
        .into()
    }
}

fn default_states() -> Vec<WrapBoxState> {
    vec![WrapBoxState::default(); WB_STATE_COUNT]
}
