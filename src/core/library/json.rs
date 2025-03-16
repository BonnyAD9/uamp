use std::{
    collections::HashSet,
    fs::{self, File},
    path::Path,
};

use log::{error, info};

use crate::{
    core::{Error, Result, TaskMsg, TaskType, config::Config, player::Player},
    env::AppCtrl,
};

use super::{
    Library, LibraryLoadResult, LoadOpts, Song, SongId,
    add_new_songs::add_new_songs,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

impl Library {
    /// Loads library according to config, returns empty library on fail
    pub fn from_config(conf: &Config) -> Self {
        if let Some(p) = conf.library_path() {
            Self::from_json(p)
        } else {
            Self::default()
        }
    }

    /// Loads the library from the given json file. Returns default library on
    /// error.
    pub fn from_json(path: impl AsRef<Path>) -> Self {
        if let Ok(file) = File::open(path.as_ref()) {
            match serde_json::from_reader(file) {
                Ok(l) => l,
                Err(e) => {
                    error!("Failed to parse library: {e}");
                    Library::default()
                }
            }
        } else {
            info!("library file {:?} doesn't exist", path.as_ref());
            Self::default()
        }
    }

    /// Starts new task that saves the library to json.
    ///
    /// The task is started only when there is any change in the library and if
    /// it is not yet running.
    ///
    /// # Errors
    /// - The save task is already running.
    pub fn start_to_default_json(
        &mut self,
        conf: &Config,
        ctrl: &mut AppCtrl,
        player: &mut Player,
    ) -> Result<()> {
        if !self.get_change() {
            return Ok(());
        }

        if ctrl.is_task_running(TaskType::LibrarySave) {
            return Error::invalid_operation()
                .msg("Cannot save library.")
                .reason("Library save is already in progress.")
                .err();
        }

        if let Some(p) = conf.library_path() {
            let path = p.clone();
            let mut me = self.clone();
            let used = player.get_ids();

            let task = move || {
                TaskMsg::LibrarySave(
                    me.write_json(path, used.iter().flatten().copied()),
                )
            };

            ctrl.add_task(TaskType::LibraryLoad, task);
        }

        self.set_change(false);

        Ok(())
    }

    /// Loads new songs on another thread.
    pub fn start_get_new_songs(
        &mut self,
        conf: &Config,
        ctrl: &mut AppCtrl,
        opts: LoadOpts,
    ) -> Result<()> {
        if ctrl.is_task_running(TaskType::LibraryLoad) {
            return Error::invalid_operation()
                .msg("Cannot load library.")
                .reason("Library load is already in progress.")
                .err();
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
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl Library {
    fn write_json<P, I>(&mut self, path: P, used: I) -> Result<Vec<SongId>>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = SongId>,
    {
        if let Some(par) = path.as_ref().parent() {
            fs::create_dir_all(par)?;
        }

        let free = self.get_free_tmp_songs(used);
        self.remove_free_tmp_songs(&free);

        serde_json::to_writer(File::create(&path)?, self)?;
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

    pub(super) fn remove_free_tmp_songs(&mut self, free: &[SongId]) {
        let songs = self.tmp_songs.vec_mut();
        for s in free {
            songs[usize::MAX - s.0].delete();
        }
        while songs.last().is_some_and(Song::is_deleted) {
            songs.pop();
        }
    }
}
