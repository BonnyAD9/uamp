use iced_core::{
    alignment::Vertical,
    font::Weight,
    Font,
    Length::{Fill, Shrink},
};

use crate::{
    app::UampApp,
    col,
    config::ConfMessage,
    core::msg::{ControlMsg, Msg},
    gui::ids::WB_SETTINGS,
    row, wrap_box,
};

use super::{
    elements::the_button,
    theme::{Container, Text},
    wid::{self, container, cursor_grad, line_text, switch, text, Element},
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
            self.items()
        ]
        .into()
    }

    fn items(&self) -> Element {
        wrap_box![
            &self.gui.wb_states[WB_SETTINGS],
            the_button("Search for new songs")
                .on_press(Msg::Control(ControlMsg::LoadNewSongs)),
            toggle(
                "Recursive search for new songs",
                self.config.recursive_search(),
                ConfMessage::RecursiveSearch
            ),
            toggle(
                "Update library on start",
                self.config.update_library_on_start(),
                ConfMessage::UpdateLibraryOnStart,
            ),
        ]
        .padding([0, 0, 0, 20])
        .spacing_y(5)
        .into()
    }
}

fn toggle<'a, M>(s: &'static str, value: bool, msg: M) -> wid::CursorGrad<'a>
where
    M: Fn(bool) -> ConfMessage + 'static,
{
    cursor_grad(
        switch(
            line_text(s)
                .vertical_alignment(Vertical::Center)
                .style(Text::NoForeground)
                .width(Shrink),
            value,
        )
        .padding([0, 0, 0, 10])
        .on_toggle(move |b| Some(Msg::Config(msg(b))))
        .width(Shrink)
        .height(Fill),
    )
    .padding([0, 10, 0, 10])
    .width(Shrink)
    .height(30)
}
