use std::mem;

use crate::core::{Result, UampApp};

use super::{LibraryUpdate, SongId};

impl UampApp {
    /// Finish up task for saving songs to json started with
    /// `start_to_default_json`.
    pub(in crate::core) fn finish_library_save_songs(
        &mut self,
        res: Result<Vec<SongId>>,
    ) -> Result<()> {
        match res {
            Ok(free) => {
                self.library.remove_free_tmp_songs(&free);
                Ok(())
            }
            Err(e) => {
                self.library.set_change(true);
                e.prepend("Failed to save library.").err()
            }
        }
    }

    /// Updates references to songs in the scope of the library with the given
    /// change and returns the change.
    pub(in crate::core) fn library_routine(&mut self) -> LibraryUpdate {
        mem::replace(&mut self.library.lib_update, LibraryUpdate::None)
    }
}
