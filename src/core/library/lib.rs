use log::{error, info};
use serde_derive::{Deserialize, Serialize};

use std::{
    cell::Cell,
    collections::HashSet,
    fs::{create_dir_all, File},
    mem,
    ops::{Index, IndexMut},
    path::Path,
};

use crate::{
    core::{
        config::Config, player::Player, Error, Result, TaskMsg, TaskType,
        UampApp,
    },
    env::AppCtrl,
    ext::alc_vec::AlcVec,
    gen_struct,
};

use super::{
    add_new_songs::add_new_songs,
    load::{LibraryLoadResult, LoadOpts},
    Filter, LibraryUpdate, Song, SongId,
};

gen_struct! {
    #[derive(Serialize, Deserialize)]
    pub Library {
        // Fields passed by reference
        songs: AlcVec<Song> { pri , pri },
        #[serde(default)]
        tmp_songs: AlcVec<Song> { pri, pri },
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
    pub fn clone_songs(&mut self) -> AlcVec<Song> {
        self.songs.clone()
    }

    /// Creates empty library
    pub fn new() -> Self {
        Library {
            songs: AlcVec::new(),
            tmp_songs: AlcVec::new(),
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
        player: &mut Player,
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
            let mut me = self.clone();
            let used = player.get_ids();

            let task = move || {
                TaskMsg::LibrarySave(
                    me.write_json(
                        path,
                        used.0
                            .iter()
                            .chain(
                                used.1
                                    .as_ref()
                                    .map(|a| a.iter())
                                    .into_iter()
                                    .flatten(),
                            )
                            .copied(),
                    ),
                )
            };

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
        let songs = self.clone_songs();
        let remove_missing =
            opts.remove_missing.unwrap_or(conf.remove_missing_on_load());

        let task = move || {
            let mut res = LibraryLoadResult {
                removed: false,
                first_new: songs.len(),
                add_policy: opts.add_to_playlist,
                sparse_new: vec![],
                songs: songs.into(),
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

    pub fn clone(&mut self) -> Self {
        Self {
            songs: self.clone_songs(),
            tmp_songs: self.tmp_songs.clone(),
            lib_update: LibraryUpdate::None,
            ghost: self.ghost.clone(),
            change: self.change.clone(),
        }
    }

    pub fn add_tmp_song(&mut self, song: Song) -> SongId {
        for (i, s) in self.tmp_songs_mut().iter_mut().enumerate() {
            if s.is_deleted() {
                *s = song;
                return SongId::tmp(i);
            }
        }

        self.tmp_songs_mut().vec_mut().push(song);
        SongId::tmp(self.tmp_songs.len() - 1)
    }

    pub fn add_tmp_path<P>(&mut self, path: P) -> Result<SongId>
    where
        P: AsRef<Path>,
    {
        Ok(self.add_tmp_song(Song::from_path(path)?))
    }
}

impl Index<SongId> for Library {
    type Output = Song;
    fn index(&self, index: SongId) -> &Self::Output {
        if index.0 >= self.songs().len() {
            let idx = index.as_tmp();
            if idx >= self.tmp_songs().len() || self.songs()[idx].is_deleted()
            {
                &self.ghost
            } else {
                &self.tmp_songs()[idx]
            }
        } else if self.songs()[index.0].is_deleted() {
            &self.ghost
        } else {
            &self.songs()[index.0]
        }
    }
}

impl IndexMut<SongId> for Library {
    fn index_mut(&mut self, index: SongId) -> &mut Song {
        if index.0 >= self.songs().len() {
            let idx = index.as_tmp();
            if idx >= self.tmp_songs().len() || self.songs()[idx].is_deleted()
            {
                &mut self.ghost
            } else {
                &mut self.tmp_songs_mut()[idx]
            }
        } else if self.songs()[index.0].is_deleted() {
            &mut self.ghost
        } else {
            &mut self.songs_mut()[index.0]
        }
    }
}

impl Index<&SongId> for Library {
    type Output = Song;
    fn index(&self, index: &SongId) -> &Self::Output {
        &self[*index]
    }
}

impl IndexMut<&SongId> for Library {
    fn index_mut(&mut self, index: &SongId) -> &mut Song {
        &mut self[*index]
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

        *self.library.songs_mut() = mem::take(&mut res.songs).into();
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

        match self.library.start_to_default_json(
            &self.config,
            ctrl,
            &mut self.player,
        ) {
            Err(Error::InvalidOperation(_)) => {}
            Err(e) => error!("Failed to start library save: {e}"),
            _ => {}
        }
    }

    pub fn finish_library_save_songs(&mut self, res: Result<Vec<SongId>>) {
        match res {
            Ok(free) => self.library.remove_free_tmp_songs(&free),
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
    fn write_json<P, I>(&mut self, path: P, used: I) -> Result<Vec<SongId>>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = SongId>,
    {
        if let Some(par) = path.as_ref().parent() {
            create_dir_all(par)?;
        }

        let free = self.get_free_tmp_songs(used);
        self.remove_free_tmp_songs(&free);

        serde_json::to_writer(File::create(path)?, self)?;
        Ok(free)
    }

    fn get_free_tmp_songs<I>(&self, used: I) -> Vec<SongId>
    where
        I: IntoIterator<Item = SongId>,
    {
        let used: HashSet<_> =
            used.into_iter().filter(|s| self.is_tmp(*s)).collect();
        (0..self.tmp_songs.len())
            .map(SongId::tmp)
            .filter(|s| !used.contains(s))
            .collect()
    }

    fn remove_free_tmp_songs(&mut self, free: &[SongId]) {
        let songs = self.tmp_songs.vec_mut();
        for s in free {
            songs[usize::MAX - s.0].delete();
        }
        while songs.last().map_or(false, Song::is_deleted) {
            songs.pop();
        }
    }

    fn is_tmp(&self, id: SongId) -> bool {
        id.0 >= self.songs.len() && usize::MAX - id.0 <= self.tmp_songs.len()
    }
}

fn default_ghost() -> Song {
    Song::invalid()
}
