use std::{
    borrow::Cow,
    cell::Cell,
    fs::{create_dir_all, File},
    path::Path,
    sync::Arc,
    time::Duration,
};

use iced_core::{
    alignment::{Horizontal, Vertical},
    font::Weight,
    Font,
    Length::{self, Fill, FillPortion, Shrink},
};
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{
    app::UampApp,
    col,
    config::Config,
    core::{
        extensions::duration_to_string,
        msg::{ComMsg, ControlMsg, Msg},
        Result,
    },
    gen_struct, grid,
    gui::widgets::icons::SvgData,
    library::{Filter, SongId},
    player::TimeStamp,
    row, text,
};

use super::{
    ids::*,
    msg::Message,
    theme::{Border, Container, CursorGrad, SvgButton, Text},
    wid::{
        self, border, button, center_x, center_y, container,
        cursor_grad, line_text, nothing, row, slider, space, svg, svg_button,
        text, wrap_box, Command, Element, GridItem, WrapBoxState,
    },
    widgets::{
        grid::SpanLen::{Fixed, Relative},
        icons,
    },
    WinMessage,
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
    Library,
    /// Current playlist page
    Playlist,
    Settings,
}

impl GuiState {
    /// Creates default gui state
    pub fn new() -> Self {
        GuiState {
            page: MainPage::Library,
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
            }
            WinMessage::Size(w, h) => {
                self.gui.window_width_set(w);
                self.gui.window_height_set(h);
            }
        }

        ComMsg::none()
    }

    /// Generates the gui
    pub fn gui(&self) -> Element {
        col![self.main_view(), self.bottom_menu()].into()
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl GuiState {
    /// Loads gui state from the given path, if it fails creates default.
    fn from_json<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
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
    fn to_json<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        if let Some(par) = path.as_ref().parent() {
            create_dir_all(par)?;
        }

        serde_json::to_writer(File::create(path)?, self)?;
        Ok(())
    }
}

impl UampApp {
    fn main_view(&self) -> Element {
        let page = match self.gui.page {
            MainPage::Library => self.library_page(),
            MainPage::Playlist => self.playlist_page(),
            MainPage::Settings => self.settings_page(),
        };

        row![self.left_menu(), page,].height(Fill).into()
    }

    fn left_menu(&self) -> Element {
        container(
            col![
                row![
                    svg(icons::UAMP).height(60).width(60),
                    text("Uamp").size(40).style(Text::Default).font(Font {
                        weight: Weight::Semibold,
                        ..Default::default()
                    }),
                ]
                .height(60),
                space(Fill, 13),
                self.left_menu_item("Library", MainPage::Library),
                self.left_menu_item("Playlist", MainPage::Playlist),
                space(Fill, Fill),
                self.left_menu_item("Settings", MainPage::Settings),
            ]
            .spacing(5),
        )
        .style(Container::Gray)
        .padding([20, 20, 20, 20])
        .width(250)
        .into()
    }

    fn left_menu_item(&self, name: &'static str, page: MainPage) -> Element {
        border(
            button(
                cursor_grad(
                    text(name)
                        .vertical_alignment(Vertical::Center)
                        .style(Text::NoForeground),
                )
                .padding([0, 0, 0, 5]),
            )
            .on_press(Msg::Gui(Message::SetPage(page)))
            .padding(0),
        )
        .height(30)
        .padding([0, 0, 0, 5])
        .style(Border::LeftRound(self.gui.page == page))
        .into()
    }

    fn library_page(&self) -> Element {
        col![
            container(row![text("Library")
                .width(300)
                .size(40)
                .vertical_alignment(Vertical::Center)
                .style(Text::Default)
                .font(Font {
                    weight: Weight::Semibold,
                    ..Default::default()
                }),],)
            .padding([5, 20, 5, 20])
            .height(80)
            .style(Container::TopGrad),
            container(self.song_list(
                self.library.filter(Filter::All).collect(),
                &self.gui.wb_states[WB_SONGS],
                false
            ))
        ]
        .height(Fill)
        .into()
    }

    fn playlist_page(&self) -> Element {
        col![
            container(row![
                text("Playlist")
                    .width(300)
                    .size(40)
                    .vertical_alignment(Vertical::Center)
                    .style(Text::Default)
                    .font(Font {
                        weight: Weight::Semibold,
                        ..Default::default()
                    }),
                nothing(),
                col![
                    nothing(),
                    Self::the_button("Shuffle", Fill)
                        .on_press(Msg::Control(ControlMsg::Shuffle))
                ]
                .width(70)
            ],)
            .padding([5, 20, 5, 20])
            .height(80)
            .style(Container::TopGrad),
            container(self.song_list(
                self.player.playlist().as_arc(),
                &self.gui.wb_states[WB_PLAYLIST],
                true
            ))
        ]
        .height(Fill)
        .into()
    }

    fn settings_page(&self) -> Element {
        col![
            container(row![text("Settings")
                .width(300)
                .size(40)
                .vertical_alignment(Vertical::Center)
                .style(Text::Default)
                .font(Font {
                    weight: Weight::Semibold,
                    ..Default::default()
                }),],)
            .padding([5, 20, 5, 20])
            .height(80)
            .style(Container::TopGrad),
            container(
                col![
                    Self::the_button("Search for new songs", 200)
                        .on_press(Msg::Control(ControlMsg::LoadNewSongs)),
                    nothing(),
                ]
                .padding([0, 0, 0, 20])
            )
            .style(Container::Dark)
        ]
        .into()
    }

    fn the_button<'a, S, L>(s: S, width: L) -> wid::Button<'a>
    where
        S: Into<Cow<'a, str>>,
        L: Into<Length>,
    {
        button(cursor_grad(
            text(s)
                .horizontal_alignment(Horizontal::Center)
                .vertical_alignment(Vertical::Center)
                .style(Text::NoForeground),
        ))
        .width(width)
        .height(30)
        .padding(0)
    }

    /// Creates a song list
    fn song_list<'a>(
        &'a self,
        songs: Arc<[SongId]>,
        state: &'a WrapBoxState,
        numbered: bool,
    ) -> Element {
        let mut items: Vec<Element> = Vec::new();

        if numbered {
            items.push(
                container(text("#").style(Text::Gray).size(14))
                    .width(50)
                    .padding([0, 0, 0, 10])
                    .into(),
            )
        }

        fn make_title<'a>(s: &'static str, portion: u16) -> Element<'a> {
            line_text(s)
                .width(FillPortion(portion))
                .style(Text::Gray)
                .elipsis("...")
                .size(14)
                .into()
        }

        items.extend([
            make_title("TITLE / ARTIST", 18),
            make_title("ALBUM / YEAR", 15),
            make_title("T / D", 2),
            make_title("LENGTH / GENRE", 3),
        ]);

        col![
            container(
                border(container(row(items)).padding([0, 40, 0, 20]))
                    .height(20)
                    .style(Border::Bot)
            )
            .height(23)
            .style(Container::Dark),
            wrap_box(
                (0..songs.len())
                    .map(|i| self.song_list_item(i, songs.clone(), numbered))
                    .collect(),
                state,
            )
            .item_height(40)
            .from_layout_style(&self.theme)
        ]
        .into()
    }

    /// Creates a song list item
    fn song_list_item(
        &self,
        song: usize,
        songs: Arc<[SongId]>,
        numbered: bool,
    ) -> Element<'static> {
        let text_style = if Some(songs[song]) == self.player.now_playing() {
            Text::Contrast
        } else {
            Text::Default
        };

        let s = &self.library[songs[song]];

        fn top_text<'a, S>(s: S, portion: u16, style: Text) -> Element<'a>
        where
            S: Into<Cow<'a, str>>,
        {
            line_text(s)
                .width(FillPortion(portion))
                .style(style)
                .elipsis("...")
                .size(14)
                .into()
        }

        fn bot_text<'a, S>(s: S, portion: u16) -> Element<'a>
        where
            S: Into<Cow<'a, str>>,
        {
            container(line_text(s).style(Text::Gray).elipsis("...").size(10))
                .width(FillPortion(portion))
                .padding([0, 0, 0, 2])
                .into()
        }

        let info = col![
            row![
                top_text(s.title().to_owned(), 18, text_style),
                top_text(s.album().to_owned(), 15, text_style),
                top_text(s.track_str(), 2, text_style),
                top_text(s.length_str(), 3, text_style),
            ]
            .height(FillPortion(3))
            .padding([4, 0, 0, 0]),
            row![
                bot_text(s.artist().to_owned(), 18),
                bot_text(s.year_str(), 15),
                bot_text(s.disc_str(), 2),
                bot_text(s.genre().to_owned(), 3),
            ]
            .height(FillPortion(2)),
        ];

        let item: Element = if numbered {
            row![
                text(song.to_string())
                    .width(50)
                    .vertical_alignment(Vertical::Center)
                    .style(Text::Gray),
                info,
            ]
            .padding([0, 10, 0, 10])
            .into()
        } else {
            info.padding([0, 10, 0, 10]).into()
        };

        border(
            button(cursor_grad(item).style(CursorGrad::Long))
                .padding(0)
                .on_press(Msg::PlaySong(song, songs))
        )
        .style(Border::SongItem)
        .into()
    }

    fn bottom_menu(&self) -> Element {
        let song = self.player.now_playing().map(|s| &self.library[s]);
        let title = song.map(|s| s.title()).unwrap_or("-");
        let artist = song.map(|s| s.artist()).unwrap_or("-");

        container(grid![
            Relative(2.), Relative(1.), Fixed(210.), Relative(1.), Relative(2.);
            Fixed(40.), Relative(1.);
            GridItem::new(
                line_text(title)
                .size(20)
                .height(25)
                .font(Font {
                    weight: Weight::Medium,
                    ..Font::default()
                })
                .elipsis("..."),
            )
            .column(0..2)
            .row(0..2)
            .padding([15, 0, 0, 15]),
            GridItem::new(
                line_text(artist)
                    .elipsis("...")
                    .size(14)
                    .style(Text::Gray)
                    .height(20)
                    .font(Font {
                        weight: Weight::Medium,
                        ..Font::default()
                    }),
            )
            .row(1)
            .padding([0, 0, 0, 20]),
            GridItem::new(self.play_menu()).column(1..4).row(0..2),
            GridItem::new(self.volume_menu()).column(4).row(0..2),
        ])
        .height(80)
        .style(Container::TopGrad)
        .into()
    }

    fn play_menu(&self) -> Element {
        let pp_svg = if self.player.is_playing() {
            icons::PAUSE
        } else {
            icons::PLAY
        };

        fn mk_button<'a>(icon: SvgData, msg: ControlMsg) -> Element<'a> {
            svg_button(icon)
                .height(30)
                .width(30)
                .padding(5)
                .on_click(Msg::Control(msg))
                .style(SvgButton::TransparentCircle(30.))
                .into()
        }

        col![
            center_x(
                row![
                    mk_button(icons::PREVIOUS, ControlMsg::PrevSong(1)),
                    mk_button(icons::REWIND, ControlMsg::Rewind(None)),
                    svg_button(pp_svg)
                        .padding(6)
                        .width(30)
                        .height(30)
                        .on_click(Msg::Control(ControlMsg::PlayPause(None)))
                        .style(SvgButton::WhiteCircle(30.)),
                    mk_button(
                        icons::FAST_FORWARD,
                        ControlMsg::FastForward(None)
                    ),
                    mk_button(icons::NEXT, ControlMsg::NextSong(1))
                ]
                .spacing(15)
                .width(Shrink)
                .padding([10, 0, 0, 0])
            )
            .height(40),
            space(Fill, 5),
            self.seek_slider()
        ]
        .into()
    }

    fn seek_slider(&self) -> Element {
        let (cur, end, slider) = match self.gui.song_timestamp {
            Some(mut ts) => {
                if let Some(d) = self.gui.seek_drag {
                    ts.current = d;
                }
                (
                    duration_to_string(ts.current, true),
                    duration_to_string(ts.total, true),
                    slider(
                        0.0..=ts.total.as_secs_f32(),
                        ts.current.as_secs_f32(),
                        |c| {
                            Msg::Gui(Message::SeekSliderMove(
                                Duration::from_secs_f32(c),
                            ))
                        },
                    )
                    .on_release(Msg::Gui(Message::SeekSliderEnd))
                    .step(0.1)
                    .width(Fill)
                    .height(10),
                )
            }
            None => (
                "--:--".to_owned(),
                "--:--".to_owned(),
                slider(0.0..=1., 0., |_| {
                    Msg::Gui(Message::SeekSliderMove(Duration::ZERO))
                })
                .width(Fill)
                .height(10),
            ),
        };

        fn mk_text<'a>(s: String) -> Element<'a> {
            text(s)
                .style(Text::Gray)
                .size(14)
                .horizontal_alignment(Horizontal::Center)
                .vertical_alignment(Vertical::Center)
                .width(50)
                .height(30)
                .into()
        }

        row![
            mk_text(cur),
            space(10, Fill),
            slider,
            space(10, Fill),
            mk_text(end),
        ]
        .height(30)
        .into()
    }

    fn volume_menu(&self) -> Element {
        let vol_style = if self.player.mute() {
            Text::Gray
        } else {
            Text::Default
        };

        grid![
            Relative(1.), Fixed(40.), Fixed(30.), Fixed(160.);
            Fixed(40.), Relative(1.);
            GridItem::new(center_y(self.mute_button()))
                    .column(1)
                    .row(1)
                    .padding([0, 10, 0, 0]),
            GridItem::new(
                text!("{:.0}", self.player.volume() * 100.)
                    .style(vol_style)
                    .width(30)
                    .vertical_alignment(Vertical::Center)
            )
            .column(2)
            .row(1),
            GridItem::new(
                center_y(
                    slider(0.0..=1., self.player.volume(), |v| {
                        Msg::Control(ControlMsg::SetVolume(v))
                    })
                    .step(0.001),
                )
                .width(150)
                .padding([0, 5])
            )
            .column(3)
            .row(1)
            .padding([0, 10, 0, 0])
        ]
        .into()
    }

    fn mute_button(&self) -> Element {
        svg_button(if self.player.mute() {
            icons::NO_VOLUME
        } else {
            icons::VOLUME
        })
        .padding(5)
        .style(SvgButton::TransparentCircle(30.))
        .width(30)
        .height(30)
        .on_click(Msg::Control(ControlMsg::Mute(None)))
        .into()
    }
}

fn default_states() -> Vec<WrapBoxState> {
    vec![WrapBoxState::default(); WB_STATE_COUNT]
}
