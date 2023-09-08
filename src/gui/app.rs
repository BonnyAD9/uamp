use std::{
    cell::Cell,
    default,
    fs::{create_dir_all, File},
    path::Path,
    sync::Arc,
    time::Duration,
};

use iced::widget::Space;
use iced_core::{
    alignment::{Horizontal, Vertical},
    font::{Family, Weight},
    Font,
    Length::{Fill, FillPortion, Shrink},
    Widget,
};
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{
    app::UampApp,
    button, col,
    config::Config,
    core::{
        extensions::duration_to_string,
        msg::{ComMsg, ControlMsg, Msg},
        Result,
    },
    gen_struct, grid,
    library::{Filter, SongId},
    player::TimeStamp,
    row, text,
};

use super::{
    ids::*,
    msg::Message,
    theme::{Border, Button, Svg, SvgButton, Text, Container, CursorGrad},
    wid::{
        self, border, button, center, center_x, center_y, container,
        line_text, nothing, slider, space, svg, svg_button, text, wrap_box,
        Command, Element, GridItem, WrapBoxState, cursor_grad, LineText, row,
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

        row![
            self.left_menu(),
            page,
        ]
        .height(Fill)
        .into()
    }

    fn left_menu(&self) -> Element {
        container(
            col![
                row![
                    svg(icons::UAMP)
                        .height(60)
                        .width(60),
                    text("Uamp")
                        .width(Fill)
                        .height(Fill)
                        .size(40)
                        .style(Text::Default)
                        .font(Font {
                            weight: Weight::Semibold,
                            ..Default::default()
                        }),
                ]
                .height(60),
                space(Fill, 23),
                self.left_menu_item("Library", MainPage::Library),
                space(Fill, 5),
                self.left_menu_item("Playlist", MainPage::Playlist),
                space(Fill, Fill),
                self.left_menu_item("Settings", MainPage::Settings),
            ]
        )
        .style(Container::Gray)
        .padding([20, 20, 20, 20])
        .width(250)
        .height(Fill)
        .into()
    }

    fn left_menu_item(&self, name: &'static str, page: MainPage) -> Element {
        border(
            button(
                cursor_grad(
                    text(name)
                        .width(Fill)
                        .height(Fill)
                        .vertical_alignment(Vertical::Center)
                        .style(Text::NoForeground)
                )
                .width(Fill)
                .height(Fill)
                .padding([0, 0, 0, 5])
            )
            .style(Button::MenuItem)
            .on_press(Msg::Gui(Message::SetPage(page)))
            .width(Fill)
            .height(Fill)
            .padding(0)
        )
        .height(30)
        .padding([0, 0, 0, 5])
        .style(Border::LeftRound(self.gui.page == page))
        .into()
    }

    fn library_page(&self) -> Element {
        col![
            container(
                row![
                    text("Library")
                        .width(300)
                        .height(Fill)
                        .size(40)
                        .vertical_alignment(Vertical::Center)
                        .style(Text::Default)
                        .font(Font {
                            weight: Weight::Semibold,
                            ..Default::default()
                        }),
                ],
            )
            .width(Fill)
            .height(Fill)
            .padding([5, 20, 5, 20])
            .height(80)
            .style(Container::TopGrad),
            container(
                self.song_list(self.library.filter(Filter::All).collect(), &self.gui.wb_states[WB_SONGS], false)
            ).style(Container::Black).width(Fill).height(Fill)
        ]
        .height(Fill)
        .into()
    }

    fn playlist_page(&self) -> Element {
        col![
            container(
                row![
                    text("Playlist")
                        .width(300)
                        .height(Fill)
                        .size(40)
                        .vertical_alignment(Vertical::Center)
                        .style(Text::Default)
                        .font(Font {
                            weight: Weight::Semibold,
                            ..Default::default()
                        }),
                    space(Fill, Fill),
                    col![
                        space(Fill, Fill),
                        button(
                            cursor_grad(
                                text("Shuffle")
                                    .horizontal_alignment(Horizontal::Center)
                                    .vertical_alignment(Vertical::Center)
                                    .width(Fill)
                                    .height(Fill)
                                    .style(Text::NoForeground)
                            )
                            .width(Fill)
                            .height(Fill)
                        )
                        .width(Fill)
                        .height(30)
                        .padding(0)
                        .on_press(Msg::Control(ControlMsg::Shuffle))
                        .style(Button::MenuItem)
                    ]
                    .width(70)
                    .height(Fill)
                ],
            )
            .width(Fill)
            .height(Fill)
            .padding([5, 20, 5, 20])
            .height(80)
            .style(Container::TopGrad),
            container(
                self.song_list(self.player.playlist().as_arc(), &self.gui.wb_states[WB_PLAYLIST], true)
            ).style(Container::Black).width(Fill).height(Fill)
        ]
        .height(Fill)
        .into()
    }

    fn settings_page(&self) -> Element {
        col![
            container(
                row![
                    text("Settings")
                        .width(300)
                        .height(Fill)
                        .size(40)
                        .vertical_alignment(Vertical::Center)
                        .style(Text::Default)
                        .font(Font {
                            weight: Weight::Semibold,
                            ..Default::default()
                        }),
                ],
            )
            .width(Fill)
            .height(Fill)
            .padding([5, 20, 5, 20])
            .height(80)
            .style(Container::TopGrad),
            container(
                col![
                    button(
                        cursor_grad(
                            text("Search for new songs")
                                .horizontal_alignment(Horizontal::Center)
                                .vertical_alignment(Vertical::Center)
                                .width(Fill)
                                .height(Fill)
                                .style(Text::NoForeground)
                        )
                        .width(Fill)
                        .height(Fill)
                    )
                    .width(200)
                    .height(30)
                    .padding(0)
                    .on_press(Msg::Control(ControlMsg::LoadNewSongs))
                    .style(Button::MenuItem),
                    space(Fill, Fill),
                ]
                .padding([0, 0, 0, 20])
            )
            .style(Container::Dark)
            .width(Fill)
            .height(Fill)
        ]
        .height(Fill)
        .into()
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
                container(
                    text("#")
                        .width(Fill)
                        .height(Fill)
                        .style(Text::Gray)
                        .size(14)
                )
                .width(50)
                .padding([0, 0, 0, 10])
                .height(Fill)
                .into()
            )
        }

        items.extend([
            line_text("TITLE / ARTIST")
                .width(FillPortion(18))
                .height(Fill)
                .style(Text::Gray)
                .elipsis("...")
                .size(14)
                .into(),
            line_text("ALBUM / YEAR")
                .width(FillPortion(15))
                .height(Fill)
                .style(Text::Gray)
                .elipsis("...")
                .size(14)
                .into(),
            line_text("T / D")
                .width(FillPortion(2))
                .height(Fill)
                .style(Text::Gray)
                .elipsis("...")
                .size(14)
                .into(),
            line_text("LENGTH / GENRE")
                .width(FillPortion(3))
                .height(Fill)
                .style(Text::Gray)
                .elipsis("...")
                .size(14)
                .into(),
        ]);

        col![
            container(
                border(
                    container(
                        row(items)
                            .height(Fill)
                            .width(Fill)
                    )
                    .height(Fill)
                    .width(Fill)
                    .padding([0, 40, 0, 20])
                ).height(20).style(Border::Bot)
            )
            .width(Fill)
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

        let info = col![
            row![
                line_text(s.title().to_owned())
                    .width(FillPortion(18))
                    .height(Fill)
                    .style(text_style)
                    .elipsis("...")
                    .size(14),
                line_text(s.album().to_owned())
                    .width(FillPortion(15))
                    .height(Fill)
                    .style(text_style)
                    .elipsis("...")
                    .size(14),
                line_text(s.track_str())
                    .width(FillPortion(2))
                    .height(Fill)
                    .style(text_style)
                    .elipsis("...")
                    .size(14),
                line_text(s.length_str())
                    .width(FillPortion(3))
                    .height(Fill)
                    .style(text_style)
                    .elipsis("...")
                    .size(14),
            ]
            .width(Fill)
            .height(FillPortion(3))
            .padding([4, 0, 0, 0]),
            row![
                container(
                    line_text(s.artist().to_owned())
                        .width(Fill)
                        .height(Fill)
                        .style(Text::Gray)
                        .elipsis("...")
                        .size(10),
                )
                .width(FillPortion(18))
                .height(Fill)
                .padding([0, 0, 0, 2]),
                container(
                    line_text(s.year_str())
                        .width(Fill)
                        .height(Fill)
                        .style(Text::Gray)
                        .elipsis("...")
                        .size(10),
                )
                .width(FillPortion(15))
                .height(Fill)
                .padding([0, 0, 0, 2]),
                container(
                    line_text(s.disc_str())
                        .width(Fill)
                        .height(Fill)
                        .style(Text::Gray)
                        .elipsis("...")
                        .size(10),
                )
                .width(FillPortion(2))
                .height(Fill)
                .padding([0, 0, 0, 2]),
                container(
                    line_text(s.genre().to_owned())
                        .width(Fill)
                        .height(Fill)
                        .style(Text::Gray)
                        .elipsis("...")
                    .size(10),
                )
                .width(FillPortion(3))
                .height(Fill)
                .padding([0, 0, 0, 2]),
            ]
            .width(Fill)
            .height(FillPortion(2)),
        ]
        .width(Fill)
        .height(Fill);

        let item: Element = if numbered {
            row![
                text(song.to_string())
                    .width(50)
                    .height(Fill)
                    .vertical_alignment(Vertical::Center)
                    .style(Text::Gray),
                info,
            ]
            .height(Fill)
            .padding([0, 10, 0, 10])
            .into()
        } else {
            info.padding([0, 10, 0, 10]).into()
        };

        border(
            button(cursor_grad(item).style(CursorGrad::Long))
                .padding(0)
                .on_press(Msg::PlaySong(song, songs))
                .height(Fill)
                .width(Fill)
                .style(Button::Item)
        )
        .style(Border::SongItem)
        .into()
    }

    fn bottom_menu(&self) -> Element {
        let song = self.player.now_playing().map(|s| &self.library[s]);
        let title = song.map(|s| s.title()).unwrap_or("-");
        let artist = song.map(|s| s.artist()).unwrap_or("-");

        border(grid![
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
                .elipsis("...")
                .width(Fill)
                .height(Fill),
            ).column(0..2).row(0..2).padding([15, 0, 0, 15]),
            GridItem::new(
                line_text(artist)
                    .elipsis("...")
                    .width(Fill)
                    .height(Fill)
                    .size(14)
                    .style(Text::Gray)
                    .height(20)
                    .font(Font {
                        weight: Weight::Medium,
                        ..Font::default()
                    }),
            ).row(1).padding([0, 0, 0, 20]),
            GridItem::new(self.play_menu()).column(1..4).row(0..2),
            GridItem::new(self.volume_menu()).column(4).row(0..2),
        ])
        .height(80)
        .width(Fill)
        .style(Border::TopGrad)
        .into()
    }

    fn play_menu(&self) -> Element {
        let pp_svg = if self.player.is_playing() {
            icons::PAUSE
        } else {
            icons::PLAY
        };
        col![
            center_x(
                row![
                    svg_button(icons::PREVIOUS)
                        .height(30)
                        .width(30)
                        .padding(5)
                        .on_click(Msg::Control(ControlMsg::PrevSong(1)))
                        .style(SvgButton::TransparentCircle(30.)),
                    space(15, Fill),
                    svg_button(icons::REWIND)
                        .height(30)
                        .width(30)
                        .padding(5)
                        .on_click(Msg::Control(ControlMsg::Rewind(None)))
                        .style(SvgButton::TransparentCircle(30.)),
                    space(15, Fill),
                    button(
                        center(
                            svg(pp_svg).width(25).height(25).style(Svg::Dark)
                        )
                        .padding(0)
                    )
                    .height(30)
                    .width(30)
                    .on_press(Msg::Control(ControlMsg::PlayPause(None)))
                    .style(Button::WhiteCircle(40.)),
                    space(15, Fill),
                    svg_button(icons::FAST_FORWARD)
                        .height(30)
                        .width(30)
                        .padding(5)
                        .on_click(Msg::Control(ControlMsg::FastForward(None)))
                        .style(SvgButton::TransparentCircle(30.)),
                    space(15, Fill),
                    svg_button(icons::NEXT)
                        .height(30)
                        .width(30)
                        .padding(5)
                        .on_click(Msg::Control(ControlMsg::NextSong(1)))
                        .style(SvgButton::TransparentCircle(30.))
                ]
                .height(40)
                .padding([10, 0, 0, 0])
            ),
            space(Fill, 5),
            self.seek_slider()
        ]
        .width(FillPortion(2))
        .into()
    }

    fn seek_slider(&self) -> Element {
        let mut ts = match self.gui.song_timestamp {
            Some(ts) => ts,
            None => {
                return row![
                    text!("--:--")
                        .style(Text::Gray)
                        .size(14)
                        .width(Fill)
                        .height(Fill)
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center)
                        .width(50)
                        .height(30),
                    space(10, Fill),
                    center_y(slider(0.0..=1., 0., |_| {
                        Msg::Gui(Message::SeekSliderMove(Duration::ZERO))
                    }))
                    .width(Fill)
                    .height(30),
                    space(10, Fill),
                    text!("--:--")
                        .style(Text::Gray)
                        .size(14)
                        .width(Fill)
                        .height(Fill)
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center)
                        .width(50)
                        .height(30),
                ]
                .into()
            }
        };

        if let Some(d) = self.gui.seek_drag {
            ts.current = d;
        }
        row![
            text!("{}", duration_to_string(ts.current, true))
                .style(Text::Gray)
                .size(14)
                .width(Fill)
                .height(Fill)
                .horizontal_alignment(Horizontal::Center)
                .vertical_alignment(Vertical::Center)
                .width(50)
                .height(30),
            space(10, Fill),
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
            space(10, Fill),
            text!("{}", duration_to_string(ts.total, true))
                .style(Text::Gray)
                .size(14)
                .width(Fill)
                .height(Fill)
                .horizontal_alignment(Horizontal::Center)
                .vertical_alignment(Vertical::Center)
                .width(50)
                .height(30),
        ]
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
            GridItem::new(center_y(self.mute_button())).column(1).row(1).padding([0, 10, 0, 0]),
            GridItem::new(center_y(text!("{:.0}", self.player.volume() * 100.).style(vol_style)).width(30)).column(2).row(1),
            GridItem::new(
                center_y(
                    slider(0.0..=1., self.player.volume(), |v| {
                        Msg::Control(ControlMsg::SetVolume(v))
                    })
                    .step(0.001),
                )
                .width(150)
                .padding([0, 5])
            ).column(3).row(1)
            .padding([0, 10, 0, 0])
        ].into()
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
    /* /// Creates the menu
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
    }*/
}

fn default_states() -> Vec<WrapBoxState> {
    vec![WrapBoxState::default(); WB_STATE_COUNT]
}
