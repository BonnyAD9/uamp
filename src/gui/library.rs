use std::{sync::Arc, cell::Cell};

use iced_core::{alignment::Vertical, font::Weight, Font, Length::Fill};
use itertools::Itertools;

use crate::{
    app::UampApp,
    col,
    library::{order::Order, Filter, SongId, LibraryUpdate},
    row,
};

use super::{
    ids::WB_SONGS,
    theme::{Container, Text},
    wid::{container, text, Element},
};

#[derive(Default)]
pub(super) struct LibState {
    view_ordering: Order,
    view_songs: Cell<Option<Arc<[SongId]>>>,
}

impl UampApp {
    pub(super) fn gui_library_lib_update(&mut self, up: LibraryUpdate) {
        if up >= LibraryUpdate::NewData {
            self.gui.lib_state.view_songs.set(None);
        }
    }

    pub(super) fn library_page(&self) -> Element {
        let songs = if let Some(s) = self.gui.lib_state.view_songs.take() {
            self.gui.lib_state.view_songs.set(Some(s.clone()));
            s
        } else {
            let mut v = self.library.filter(Filter::All).collect_vec();
            self.gui.lib_state.view_ordering.vec(&self.library, &mut v);
            let v: Arc<[_]> = v.into();
            self.gui.lib_state.view_songs.set(Some(v.clone()));
            v
        };

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
                songs,
                &self.gui.wb_states[WB_SONGS],
                false
            ))
        ]
        .height(Fill)
        .into()
    }
}
