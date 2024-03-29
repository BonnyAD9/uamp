use std::{borrow::Cow, sync::Arc};

use iced_core::{
    alignment::{Horizontal, Vertical},
    Length::{Fill, FillPortion, Shrink},
};

use crate::{
    app::UampApp,
    col,
    core::msg::Msg,
    gui::{
        app::MainPage,
        msg::Message,
        theme::{Border, Button, Container, CursorGrad, Text},
        wid::{
            self, border, button, container, cursor_grad, image, line_text,
            row, space, svg, text, wrap_box, Element, WrapBoxState,
        },
        widgets::icons,
    },
    library::{
        order::{Order, OrderField},
        SongId,
    },
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
                container(
                    text("#")
                        .horizontal_alignment(Horizontal::Right)
                        .style(Text::Gray)
                        .size(14),
                )
                .width(50)
                .padding([0, 10, 0, 10])
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
            space(35, Fill).into(),
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
                    .filter_map(|i| self.song_list_item(
                        i,
                        songs.clone(),
                        numbered
                    ))
                    .collect(),
                state,
            )
            .padding([0, 0, 0, 20])
            .item_height(40)
        ]
        .into()
    }

    /// Creates a song list
    pub(super) fn ordered_song_list<'a, F>(
        &'a self,
        songs: Arc<[SongId]>,
        state: &'a WrapBoxState,
        numbered: bool,
        cur_order: Order,
        on_order: F,
    ) -> Element
    where
        F: Fn(Order) -> Msg,
    {
        let mut items: Vec<Element> = Vec::new();

        if numbered {
            items.push(
                container(
                    text("#")
                        .horizontal_alignment(Horizontal::Center)
                        .style(Text::Gray)
                        .size(14),
                )
                .width(50)
                .padding([0, 10, 0, 10])
                .into(),
            )
        }

        fn make_title<'a, F>(
            ts: &'static str,
            bs: &'static str,
            to: Order,
            bo: Order,
            cur: Order,
            msg: F,
            portion: u16,
        ) -> Element<'a>
        where
            F: Fn(Order) -> Msg,
        {
            let (t_style, to) = if to.field == cur.field {
                (Button::SelectedGrayHover, to.set_rev(!cur.reverse))
            } else {
                (Button::GrayHover, to)
            };

            let (b_style, bo) = if bo.field == cur.field {
                (Button::SelectedGrayHover, bo.set_rev(!cur.reverse))
            } else {
                (Button::GrayHover, bo)
            };

            row![
                button(
                    line_text(ts)
                        .width(Shrink)
                        .style(Text::NoForeground)
                        .elipsis("...")
                        .size(14)
                )
                .padding(0)
                .width(Shrink)
                .on_press(msg(to))
                .style(t_style),
                line_text(" / ").width(Shrink).style(Text::Gray).size(14),
                button(
                    line_text(bs)
                        .width(Shrink)
                        .style(Text::NoForeground)
                        .elipsis("...")
                        .size(14)
                )
                .padding(0)
                .width(Shrink)
                .on_press(msg(bo))
                .style(b_style),
            ]
            .width(FillPortion(portion))
            .into()
        }

        let simple_sort = self.config.simple_sorting();

        items.extend([
            space(35, Fill).into(),
            make_title(
                "TITLE",
                "ARTIST",
                Order::new(OrderField::Title, simple_sort),
                Order::new(OrderField::Artist, simple_sort),
                cur_order,
                &on_order,
                18,
            ),
            make_title(
                "ALBUM",
                "YEAR",
                Order::new(OrderField::Album, simple_sort),
                Order::new(OrderField::Year, simple_sort),
                cur_order,
                &on_order,
                15,
            ),
            make_title(
                "T",
                "D",
                Order::new(OrderField::Track, simple_sort),
                Order::new(OrderField::Disc, simple_sort),
                cur_order,
                &on_order,
                2,
            ),
            make_title(
                "LENGTH",
                "GENRE",
                Order::new(OrderField::Length, simple_sort),
                Order::new(OrderField::Genre, simple_sort),
                cur_order,
                &on_order,
                3,
            ),
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
                    .filter_map(|i| self.song_list_item(
                        i,
                        songs.clone(),
                        numbered
                    ))
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
    ) -> Option<Element<'static>> {
        let text_style = if Some(songs[song]) == self.player.now_playing() {
            Text::Contrast
        } else {
            Text::Default
        };

        let s = &self.library[songs[song]];

        if s.is_deleted() {
            return None;
        }

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

        let img: Element = self
            .library
            .get_small_image(songs[song])
            .map(|i| image(i).width(Fill).height(Fill).into())
            .unwrap_or_else(|| {
                svg(icons::IMG_PLACEHOLDER).width(Fill).height(Fill).into()
            });

        let item: Element = if numbered {
            row![
                line_text((song + 1).to_string())
                    .width(50)
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Center)
                    .style(Text::Gray),
                container(img)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .width(35)
                    .height(40)
                    .padding([5, 5, 5, 0]),
                info,
            ]
            .padding([0, 10, 0, 10])
            .into()
        } else {
            row![
                container(img)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .width(35)
                    .height(40)
                    .padding([5, 5, 5, 0]),
                info
            ]
            .padding([0, 10, 0, 10])
            .into()
        };

        Some(
            border(
                button(cursor_grad(item).style(CursorGrad::Long))
                    .padding(0)
                    .on_press(Msg::PlaySong(song, songs)),
            )
            .style(Border::SongItem)
            .into(),
        )
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
