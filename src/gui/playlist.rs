use iced_core::{
    alignment::Vertical,
    font::Weight,
    Font,
    Length::{Fill, Shrink},
};

use crate::{
    app::UampApp,
    col,
    core::msg::{ControlMsg, Msg},
    row,
};

use super::{
    elements::the_button,
    ids::WB_PLAYLIST,
    theme::{Container, Text},
    wid::{container, nothing, space, text, Element},
};

impl UampApp {
    pub(super) fn playlist_page(&self) -> Element {
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
                    space(Shrink, Fill),
                    the_button("Shuffle")
                        .on_press(Msg::Control(ControlMsg::Shuffle))
                ]
                .width(Shrink)
            ])
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
}
