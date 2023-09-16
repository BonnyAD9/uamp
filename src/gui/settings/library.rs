use iced_core::Length::{Shrink, Fill};

use crate::{app::UampApp, gui::{wid::{Element, container, space}, ids::WB_SETTINGS_LIBRARY, elements::the_button, widgets::icons}, wrap_box, core::msg::{Msg, ControlMsg}, config::ConfMessage};

use super::{elements::{toggle, title, delete_list, add_input, EmptyBehaviour}, SetMessage};

impl UampApp {
    pub(super) fn library(&self) -> Element {
        wrap_box![
            &self.gui.wb_states[WB_SETTINGS_LIBRARY],

            //===================================<< Search for new songs button
            the_button("Search for new songs")
                .on_press(Msg::Control(ControlMsg::LoadNewSongs)),

            //=========================<< Recursive search for new songs toggle
            toggle(
                "Recursive search for new songs",
                self.config.recursive_search(),
                ConfMessage::RecursiveSearch
            ),

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
