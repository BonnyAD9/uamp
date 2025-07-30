use std::{
    collections::HashSet,
    fs::{self, File},
    path::Path,
};

use log::{error, info};

use crate::core::{
    AppCtrl, Error, Job, JobMsg, Jobs, Msg, Result, config::Config,
    player::Player,
};

use super::{Library, Song, SongId};

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

    pub fn save_to_default_json(
        &mut self,
        conf: &Config,
        player: &mut Player,
    ) -> Result<()> {
        if let Some(p) = conf.library_path() {
            let used = player.get_ids();
            self.write_json(p, used.iter().flatten().copied())?;
        }
        Ok(())
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
        jobs: &mut Jobs,
    ) -> Result<()> {
        if !self.get_change() {
            return Ok(());
        }

        if jobs.is_running(Job::LIBRARY_SAVE) {
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
                Msg::Job(JobMsg::LibrarySave(
                    me.write_json(path, used.iter().flatten().copied()),
                ))
            };

            ctrl.task(async move {
                match tokio::task::spawn_blocking(task).await {
                    Ok(r) => r,
                    Err(e) => Msg::Job(JobMsg::LibrarySave(Err(e.into()))),
                }
            });

            jobs.run(Job::LIBRARY_SAVE);
        }

        self.set_change(false);

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
