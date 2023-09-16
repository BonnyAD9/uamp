use iced_core::Length::{Fill, Shrink};

use crate::{
    app::UampApp,
    config::ConfMessage,
    core::msg::{ControlMsg, Msg},
    gui::{
        elements::the_button,
        ids::WB_SETTINGS_LIBRARY,
        wid::{container, mouse_int, space, Element},
        widgets::icons,
        GuiMessage,
    },
    wrap_box,
};

use super::{
    elements::{add_input, delete_list, title, toggle, EmptyBehaviour},
    help::{SEARCH_FOR_NEW_SONGS, RECURSIVE_SEARCH_FOR_NEW_SONGS},
    SetMessage,
};

impl UampApp {
    pub(super) fn library(&self) -> Element {
        wrap_box![
            &self.gui.wb_states[WB_SETTINGS_LIBRARY],
            //===================================<< Search for new songs button
            mouse_int(
                the_button("Search for new songs")
                    .on_press(Msg::Control(ControlMsg::LoadNewSongs)),
            )
            .on_mouse_enter(Msg::Gui(GuiMessage::Setings(
                SetMessage::ShowHelp(&SEARCH_FOR_NEW_SONGS)
            ))),
            //=========================<< Recursive search for new songs toggle
            mouse_int(
                toggle(
                    "Recursive search for new songs",
                    self.config.recursive_search(),
                    ConfMessage::RecursiveSearch
                )
            )
            .on_mouse_enter(Msg::Gui(GuiMessage::Setings(
                SetMessage::ShowHelp(&RECURSIVE_SEARCH_FOR_NEW_SONGS)
            ))),
            //================================<< Update library on start toggle
            toggle(
                "Update library on start",
                self.config.update_library_on_start(),
                ConfMessage::UpdateLibraryOnStart,
            ),
            //===============================<< Library search paths list + add
            title("Library search paths"),
            delete_list(
                self.config
                    .search_paths()
                    .iter()
                    .map(|p| p.to_string_lossy()),
                ConfMessage::RemoveSearchPath
            ),
            container(add_input(
                "path",
                &self.gui.set_state.search_path_state,
                SetMessage::SearchPathInput,
                |_| true,
                SetMessage::SearchPathConfirm,
                icons::ADD,
                EmptyBehaviour::Ignore,
            ))
            .width(400)
            .height(Shrink)
            .padding([0, 0, 0, 25]),
            //====================================<< Song extensions list + add
            title("Song extensions"),
            delete_list(
                self.config.audio_extensions().iter().map(|p| p.into()),
                ConfMessage::RemoveAudioExtension
            ),
            container(add_input(
                "extension",
                &self.gui.set_state.extension_state,
                SetMessage::ExtensionInput,
                |s| !s.find('.').is_some(),
                SetMessage::ExtensionConfirm,
                icons::ADD,
                EmptyBehaviour::Ignore,
            ))
            .width(200)
            .height(Shrink)
            .padding([0, 0, 0, 25]),
            space(Fill, 20),
        ]
        .padding([0, 0, 0, 20])
        .spacing_y(5)
        .into()
    }
}
