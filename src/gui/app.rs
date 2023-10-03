use std::{
    cell::Cell,
    fs::{create_dir_all, File},
    path::Path,
    time::Duration,
};

use iced_core::{
    alignment::{Horizontal, Vertical},
    font::Weight,
    Font,
    Length::{Fill, Shrink},
};
use log::{error, info};
use raplay::Timestamp;
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
    library::LibraryUpdate,
    row, text,
};

use super::{
    ids::*,
    library::LibState,
    msg::Message,
    settings::SetState,
    theme::{Container, SvgButton, Text},
    wid::{
        center_x, center_y, container, image, line_text, slider, space, svg,
        svg_button, text, Command, Element, GridItem, WrapBoxState,
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
        pub (super) page: MainPage,
        /// States of WrapBoxes
        #[serde(skip, default = "default_states")]
        pub (super) wb_states: Vec<WrapBoxState>,
        #[serde(skip)]
        pub (super) set_state: SetState,
        #[serde(skip)]
        pub (super) lib_state: LibState,

        #[serde(skip)]
        song_timestamp: Option<Timestamp>,
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
            set_state: Default::default(),
            lib_state: Default::default(),
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
            Message::Settings(msg) => return self.settings_event(msg),
            Message::Library(msg) => return self.gui_library_event(msg),
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

    pub fn gui_lib_update(&mut self, up: LibraryUpdate) {
        self.gui_library_lib_update(up);
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
        const WIDTH: f32 = 250.0;

        let img: Element = self
            .player
            .now_playing()
            .and_then(|s| self.library.get_image(s))
            .map(|i| image(i).height(WIDTH).into())
            .unwrap_or_else(|| {
                svg(icons::IMG_PLACEHOLDER).height(WIDTH).into()
            });

        container(col![
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
            .spacing(5)
            .padding([20, 20, 15, 20]),
            img
        ])
        .style(Container::Gray)
        .width(WIDTH)
        .into()
    }

    fn bottom_menu(&self) -> Element {
        let song = self.player.now_playing().map(|s| &self.library[s]);
        let (title, artist) = if let Some(s) = song {
            if s.is_deleted() {
                ("-", "-")
            } else {
                (s.title(), s.artist())
            }
        } else {
            ("-", "-")
        };

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
                    mk_button(icons::PREVIOUS, ControlMsg::PrevSong(None)),
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
                    if self.config.show_remaining_time() {
                        format!(
                            "-{}",
                            duration_to_string(ts.total - ts.current, true)
                        )
                    } else {
                        duration_to_string(ts.total, true)
                    },
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
                    .step(0.01)
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
            icons::no_volume(self.player.volume())
        } else {
            icons::volume(self.player.volume())
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
