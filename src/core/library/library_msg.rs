use std::mem;

use log::error;

use crate::core::{Result, UampApp};

use super::{LibraryUpdate, SongId};

impl UampApp {
    /// Finish up task for saving songs to json started with
    /// `start_to_default_json`.
    pub(in crate::core) fn finish_library_save_songs(
        &mut self,
        res: Result<Vec<SongId>>,
    ) {
        match res {
            Ok(free) => self.library.remove_free_tmp_songs(&free),
            Err(e) => {
                error!("Failed to save library: {}", e.log());
                self.library.set_change(true);
            }
        }
    }

    /// Updates references to songs in the scope of the library with the given
    /// change and returns the change.
    pub(in crate::core) fn library_update(&mut self) -> LibraryUpdate {
        mem::replace(&mut self.library.lib_update, LibraryUpdate::None)
    }
}
