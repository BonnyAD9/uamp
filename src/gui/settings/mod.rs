use crate::{app::UampApp, col, core::msg::ComMsg, row, wrap_box};

pub mod elements;
mod help;
mod hotkeys;
mod library;
mod other;
mod playback;
mod server;
mod state;

use iced_core::{alignment::Vertical, font::Weight, Font, Length::Shrink};
pub use state::{SetMessage, SetState};

use self::state::Category;

use super::{
    ids::WB_SETTINGS_HELP,
    theme::Container,
    wid::{container, line_text, nothing, Element},
    GuiMessage,
};

impl UampApp {
    pub(super) fn settings_event(&mut self, msg: SetMessage) -> ComMsg {
        self.settings_event_inner(msg)
    }

    pub(super) fn settings_page(&self) -> Element {
        let mut content = match self.gui.set_state.category {
            Category::Library => self.library(),
            Category::Playback => self.playback(),
            Category::Hotkeys => self.hotkeys(),
            Category::Server => self.server(),
            Category::Other => self.other(),
        };

        if let Some(h) = self.gui.set_state.help {
            content = row![
                content,
                wrap_box![
                    &self.gui.wb_states[WB_SETTINGS_HELP],
                    h.get_element()
                ]
            ]
            .into()
        }

        col![
            container(row![
                line_text("Settings")
                    .width(Shrink)
                    .size(40)
                    .vertical_alignment(Vertical::Center)
                    .font(Font {
                        weight: Weight::Semibold,
                        ..Default::default()
                    }),
                col![nothing(), self.categories(),].padding([0, 0, 10, 10])
            ])
            .padding([5, 20, 5, 20])
            .height(80)
            .style(Container::TopGrad),
            content
        ]
        .into()
    }

    fn categories(&self) -> Element {
        row![
            self.category_button("Library", Category::Library),
            self.category_button("Playback", Category::Playback),
            self.category_button("Hotkeys", Category::Hotkeys),
            self.category_button("Server", Category::Server),
            self.category_button("Other", Category::Other),
            nothing(),
        ]
        .height(40)
        .spacing(5)
        .padding([0, 0, 5, 0])
        .into()
    }

    fn category_button(&self, s: &'static str, category: Category) -> Element {
        self.top_menu_item(
            s,
            GuiMessage::Setings(SetMessage::SetCategory(category)),
            self.gui.set_state.category == category,
        )
    }
}
