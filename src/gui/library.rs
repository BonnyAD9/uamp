use iced_core::{alignment::Vertical, font::Weight, Font, Length::Fill};

use crate::{app::UampApp, col, library::Filter, row};

use super::{
    ids::WB_SONGS,
    theme::{Container, Text},
    wid::{container, text, Element},
};

impl UampApp {
    pub(super) fn library_page(&self) -> Element {
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
}
