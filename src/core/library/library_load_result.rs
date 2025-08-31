use std::{collections::HashSet, fmt::Debug, mem};

use crate::core::{
    AppCtrl, Error, Job, JobMsg, Msg, Result, UampApp,
    library::{LoadOpts, add_new_songs::add_new_songs},
    player::AddPolicy,
};

use super::{LibraryUpdate, Song, SongId};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Result of library load on another thread
pub struct LibraryLoadResult {
    /// True if any songs were removed from the library.
    pub(super) removed: bool,
    /// The new library contents (all songs not only the new ones)
    pub(super) songs: Vec<Song>,
    /// Determines what to do with the new songs.
    pub(super) add_policy: Option<AddPolicy>,
    /// Index of first new song.
    pub(super) first_new: usize,
    /// New songs with index smaller than [`LibraryLoadResult::first_new`]
    pub(super) sparse_new: Vec<SongId>,
}

impl LibraryLoadResult {
    /// Checks if there is any change in the library.
    pub fn any_change(&self) -> bool {
        self.removed
            || self.first_new != self.songs.len()
            || !self.sparse_new.is_empty()
    }
}

/// Less verbose implementation that doesn't list all the songs
impl Debug for LibraryLoadResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LibraryLoadResult")
            .field("removed", &self.removed)
            .field("songs.len", &self.songs.len())
            .field("add_policy", &self.add_policy)
            .field("first_new", &self.first_new)
            .field("sparse_new.len", &self.sparse_new.len())
            .finish()
    }
}

impl UampApp {
    /// Loads new songs on another thread.
    pub fn start_get_new_songs(
        &mut self,
        ctrl: &mut AppCtrl,
        opts: LoadOpts,
    ) -> Result<()> {
        if self.jobs.is_running(Job::LIBRARY_LOAD) {
            return Error::invalid_operation()
                .msg("Cannot load library.")
                .reason("Library load is already in progress.")
                .err();
        }

        let conf = self.config.clone();
        let songs = self.library.clone_songs();
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
                Msg::Job(JobMsg::LibraryLoad(Ok(Some(res))))
            } else {
                Msg::Job(JobMsg::LibraryLoad(Ok(None)))
            }
        };

        self.jobs.run(Job::LIBRARY_LOAD);

        ctrl.task(async move {
            match tokio::task::spawn_blocking(task).await {
                Ok(r) => r,
                Err(e) => Msg::Job(JobMsg::LibraryLoad(Err(e.into()))),
            }
        });

        Ok(())
    }

    /// Finishes loading songs started with `start_get_new_songs`.
    ///
    /// This will also start to save the library to json if there is any
    /// change.
    pub(in crate::core) fn finish_library_load(
        &mut self,
        ctrl: &mut AppCtrl,
        res: Option<LibraryLoadResult>,
    ) -> Result<()> {
        self.jobs.finish(Job::LIBRARY_LOAD);

        let Some(mut res) = res else {
            return Ok(());
        };

        let old_cnt = self.library.songs().len();

        *self.library.songs_mut() = mem::take(&mut res.songs).into();
        if res.removed {
            if res.first_new < old_cnt || !res.sparse_new.is_empty() {
                // New songs ids replaced old song ids.
                self.library.update(LibraryUpdate::ReplaceData);
                let new_ids: HashSet<_> = res.sparse_new.iter().collect();
                self.id_replace(|s, lib| {
                    !lib.is_tmp(s)
                        && (s.as_norm() >= res.first_new
                            || new_ids.contains(&s))
                });
            } else {
                self.library.update(LibraryUpdate::RemoveData);
            }
        } else {
            self.library.update(LibraryUpdate::NewData);
        }

        self.player.add_songs(
            || {
                (res.first_new..self.library.songs().len())
                    .map(SongId::norm)
                    .chain(res.sparse_new.iter().copied())
            },
            res.add_policy,
        );

        self.client_update_set_all();

        match self.library.start_to_default_json(
            &self.config,
            ctrl,
            &mut self.player,
            &mut self.jobs,
        ) {
            Err(Error::InvalidOperation(_)) => Ok(()),
            Err(e) => e
                .prepend("Failed to start library save after library load.")
                .err(),
            _ => Ok(()),
        }
    }
}
