use log::{error, info};
use serde_derive::{Deserialize, Serialize};

use std::{
    cell::Cell, collections::{BTreeSet, HashMap, HashSet}, fs::{create_dir_all, read_dir, File}, mem, ops::{Index, IndexMut}, path::Path
};

use crate::{
    app::UampApp,
    config::Config,
    core::{command::AppCtrl, Error, Result},
    gen_struct,
    sync::tasks::{TaskMsg, TaskType},
};

use super::{
    add_new_songs::add_new_songs, load::{LibraryLoadResult, LoadOpts}, Filter, LibraryUpdate, Song, SongId
};

gen_struct! {
    #[derive(Serialize, Deserialize)]
    pub Library {
        // Fields passed by reference
        songs: Vec<Song> { pri , pri },
        // albums: Vec<SongId> { pri, pri },
        ; // Fields passed by value
        ; // Other fields
        /// invalid song
        #[serde(skip, default = "default_ghost")]
        ghost: Song,
        #[serde(skip)]
        lib_update: LibraryUpdate,
        ; // attributes for the auto field
        #[serde(skip)]
    }
}

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

impl Library {
    /// Creates empty library
    pub fn new() -> Self {
        Library {
            songs: Vec::new(),
            lib_update: LibraryUpdate::None,
            change: Cell::new(true),
            ghost: Song::invalid(),
        }
    }

    pub fn update(&mut self, up: LibraryUpdate) {
        if up > self.lib_update {
            self.lib_update = up;
        }
    }

    /// Loads library according to config, returns empty library on fail
    pub fn from_config(conf: &Config) -> Self {
        if let Some(p) = conf.library_path() {
            Self::from_json(p)
        } else {
            Self::default()
        }
    }

    /// Filters songs in the library
    pub fn filter<'a>(
        &'a self,
        filter: Filter,
    ) -> Box<dyn Iterator<Item = SongId> + 'a> {
        match filter {
            Filter::All => Box::new(
                (0..self.songs().len())
                    .map(SongId)
                    .filter(|s| !self[*s].is_deleted()),
            ),
        }
    }

    pub fn start_to_default_json(
        &mut self,
        conf: &Config,
        ctrl: &mut AppCtrl,
    ) -> Result<()> {
        if !self.change.get() {
            return Ok(());
        }

        if ctrl.is_task_running(TaskType::LibrarySave) {
            return Err(Error::InvalidOperation(
                "Library load is already in progress",
            ));
        }

        if let Some(p) = conf.library_path() {
            let path = p.clone();
            let me = self.clone();

            let task = move || TaskMsg::LibrarySave(me.to_json(path));

            ctrl.add_task(TaskType::LibraryLoad, task);
        }

        self.change.set(false);

        Ok(())
    }

    /// Loads the library from the given json file. Returns default library on
    /// error.
    pub fn from_json(path: impl AsRef<Path>) -> Self {
        if let Ok(file) = File::open(path.as_ref()) {
            match serde_json::from_reader(file) {
                Ok(l) => l,
                Err(e) => {
                    error!("Failed to load library: {e}");
                    Library::default()
                }
            }
        } else {
            info!("library file {:?} doesn't exist", path.as_ref());
            Self::default()
        }
    }

    /// Loads new songs on another thread
    pub fn start_get_new_songs(
        &mut self,
        conf: &Config,
        ctrl: &mut AppCtrl,
        opts: LoadOpts,
    ) -> Result<()> {
        if ctrl.is_task_running(TaskType::LibraryLoad) {
            return Err(Error::InvalidOperation(
                "Library load is already in progress",
            ));
        }

        let conf = conf.clone();
        let songs = self.songs().clone();
        let remove_missing =
            opts.remove_missing.unwrap_or(conf.remove_missing_on_load());

        let task = move || {
            let mut res = LibraryLoadResult {
                removed: false,
                first_new: songs.len(),
                add_policy: opts.add_to_playlist,
                sparse_new: vec![],
                songs,
            };

            add_new_songs(&mut res, &conf, remove_missing);

            if res.any_change() {
                TaskMsg::LibraryLoad(Ok(Some(res)))
            } else {
                TaskMsg::LibraryLoad(Ok(None))
            }
        };
        ctrl.add_task(TaskType::LibraryLoad, task);

        Ok(())
    }
}

impl Index<SongId> for Library {
    type Output = Song;
    fn index(&self, index: SongId) -> &Self::Output {
        if index.0 >= self.songs().len() {
            &self.ghost
        } else {
            let r = &self.songs()[index.0];
            if r.is_deleted() {
                &self.ghost
            } else {
                r
            }
        }
    }
}

impl IndexMut<SongId> for Library {
    fn index_mut(&mut self, index: SongId) -> &mut Song {
        if index.0 >= self.songs().len() || self.songs()[index.0].is_deleted()
        {
            &mut self.ghost
        } else {
            &mut self.songs_mut()[index.0]
        }
    }
}

impl Default for Library {
    fn default() -> Self {
        Library::new()
    }
}

impl UampApp {
    pub fn finish_library_load(
        &mut self,
        ctrl: &mut AppCtrl,
        res: Result<Option<LibraryLoadResult>>,
    ) {
        let mut res = match res {
            Ok(Some(res)) => res,
            Ok(None) => return,
            Err(e) => {
                error!("Failed to load new songs: {e}");
                return;
            }
        };

        *self.library.songs_mut() = mem::take(&mut res.songs);
        if res.removed {
            self.library.update(LibraryUpdate::RemoveData);
        } else {
            self.library.update(LibraryUpdate::NewData);
        }

        if let Some(p) = res.add_policy {
            self.player.add_songs(
                (res.first_new..self.library.songs().len())
                    .map(SongId)
                    .chain(res.sparse_new),
                p,
            );
        };

        match self.library.start_to_default_json(&self.config, ctrl) {
            Err(Error::InvalidOperation(_)) => {}
            Err(e) => error!("Failed to start library save: {e}"),
            _ => {}
        }
    }

    pub fn finish_library_save_songs(&mut self, res: Result<()>) {
        match res {
            Ok(()) => {}
            Err(e) => {
                error!("Failed to save library: {e}");
                self.library.change.set(true);
            }
        }
    }

    pub fn library_lib_update(&mut self) -> LibraryUpdate {
        mem::replace(&mut self.library.lib_update, LibraryUpdate::None)
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl Library {
    /// Saves the library to the specified path.
    ///
    /// # Errors
    /// - Fails to create the parent directory
    /// - Fails to write file
    /// - Fails to serialize
    fn to_json(&self, path: impl AsRef<Path>) -> Result<()> {
        if let Some(par) = path.as_ref().parent() {
            create_dir_all(par)?;
        }

        serde_json::to_writer(File::create(path)?, self)?;
        Ok(())
    }
}

impl Clone for Library {
    fn clone(&self) -> Self {
        Self {
            songs: self.songs.clone(),
            lib_update: LibraryUpdate::None,
            ghost: self.ghost.clone(),
            change: self.change.clone(),
        }
    }
}

fn default_ghost() -> Song {
    Song::invalid()
}
