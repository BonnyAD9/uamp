use std::{borrow::Cow, sync::Arc};

use iced_core::{
    alignment::{Horizontal, Vertical},
    Length::{FillPortion, Shrink},
};

use crate::{
    app::UampApp,
    col,
    core::msg::Msg,
    gui::{
        app::MainPage,
        msg::Message,
        theme::{Border, Container, CursorGrad, Text},
        wid::{
            self, border, button, container, cursor_grad, line_text, row,
            text, wrap_box, Element, WrapBoxState,
        },
    },
    library::SongId,
    row,
};

impl UampApp {
    pub(super) fn left_menu_item(
        &self,
        name: &'static str,
        page: MainPage,
    ) -> Element {
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

    pub(super) fn top_menu_item(
        &self,
        name: &'static str,
        click: Message,
        is_selected: bool,
    ) -> Element {
        border(
            button(
                cursor_grad(
                    line_text(name)
                        .vertical_alignment(Vertical::Center)
                        .style(Text::NoForeground)
                        .width(Shrink),
                )
                .width(Shrink)
                .padding([0, 5, 0, 5]),
            )
            .on_press(Msg::Gui(click))
            .width(Shrink)
            .padding(0),
        )
        .width(Shrink)
        .height(35)
        .padding([0, 0, 5, 0])
        .style(Border::BotRound(is_selected))
        .into()
    }

    /// Creates a song list
    pub(super) fn song_list<'a>(
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
            .padding([0, 0, 0, 20])
            .item_height(40)
        ]
        .into()
    }

    /// Creates a song list item
    pub(super) fn song_list_item(
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
                text((song + 1).to_string())
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
                .on_press(Msg::PlaySong(song, songs)),
        )
        .style(Border::SongItem)
        .into()
    }
}

pub(super) fn the_button<'a, S>(s: S) -> wid::Button<'a>
where
    S: Into<Cow<'a, str>>,
{
    button(
        cursor_grad(
            line_text(s)
                .horizontal_alignment(Horizontal::Center)
                .vertical_alignment(Vertical::Center)
                .style(Text::NoForeground)
                .width(Shrink),
        )
        .width(Shrink)
        .padding([0, 10, 0, 10]),
    )
    .width(Shrink)
    .height(30)
    .padding(0)
}
