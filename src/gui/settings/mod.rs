use crate::{app::UampApp, core::msg::ComMsg, col, row};

mod state;
mod library;
mod other;
mod playback;
mod server;
mod hotkeys;
pub mod elements;

use iced_core::{alignment::Vertical, Font, font::Weight, Length::Shrink};
pub use state::{SetMessage, SetState};

use self::state::Category;

use super::{wid::{Element, container, line_text, nothing}, theme::Container, GuiMessage};

impl UampApp {
    pub(super) fn settings_event(&mut self, msg: SetMessage) -> ComMsg {
        self.settings_event_inner(msg)
    }


    pub(super) fn settings_page(&self) -> Element {
        let content = match self.gui.set_state.category {
            Category::Library => self.library(),
            Category::Playback => self.playback(),
            Category::Hotkeys => self.hotkeys(),
            Category::Server => self.server(),
            Category::Other => self.other(),
        };

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
                col![
                    nothing(),
                    self.categories(),
                ]
                .padding([0, 0, 10, 10])
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
            self.gui.set_state.category == category
        )
    }
}
