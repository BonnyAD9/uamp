use iced_core::{alignment::Vertical, font::Weight, Font};

use crate::{
    app::UampApp,
    col,
    core::msg::{ControlMsg, Msg},
    row,
};

use super::{
    elements::the_button,
    theme::{Container, Text},
    wid::{container, nothing, text, Element},
};

impl UampApp {
    pub(super) fn settings_page(&self) -> Element {
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
                    the_button("Search for new songs", 200)
                        .on_press(Msg::Control(ControlMsg::LoadNewSongs)),
                    nothing(),
                ]
                .padding([0, 0, 0, 20])
            )
            .style(Container::Dark)
        ]
        .into()
    }
}
