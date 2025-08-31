use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs::{self, DirEntry, FileType},
    path::PathBuf,
};

use log::error;

use crate::core::{Result, config::Config};

use super::{LibraryLoadResult, Song, SongId};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Adds new songs to the given vector of songs
pub(super) fn add_new_songs(
    res: &mut LibraryLoadResult,
    conf: &Config,
    remove_missing: bool,
) {
    let mut state = State::new(res, conf, remove_missing);
    state.init();
    state.load();
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

struct State<'a> {
    conf: &'a Config,
    remove_missing: bool,
    // All the songs in the library, this is the main result.
    songs: &'a mut Vec<Song>,
    // Ids of new songs that are not at the end of `songs`.
    sparse_new: &'a mut Vec<SongId>,
    // Index of the first song that is new at the end of `songs`.
    first_new: &'a mut usize,
    // True if some songs were removed from `songs`.
    any_removed: &'a mut bool,
    // Directories to visit.
    paths: BTreeSet<PathBuf>,
    // Already visited directories.
    searched: HashSet<PathBuf>,
    // Indexes of all deleted songs in `songs`.
    empty: Vec<usize>,
    // Maps song path to its id.
    id_map: HashMap<PathBuf, usize>,
}

macro_rules! err_cont {
    ($e:expr, $err:ident => $msg:literal $(,)?) => {
        match $e {
            Ok(v) => v,
            Err($err) => {
                error!($msg);
                continue;
            }
        }
    };
}

impl<'a> State<'a> {
    fn new(
        res: &'a mut LibraryLoadResult,
        conf: &'a Config,
        remove_missing: bool,
    ) -> Self {
        let paths = conf
            .search_paths()
            .iter()
            .flat_map(|p| p.canonicalize())
            .collect();

        Self {
            conf,
            remove_missing,
            songs: &mut res.songs,
            sparse_new: &mut res.sparse_new,
            first_new: &mut res.first_new,
            any_removed: &mut res.removed,
            paths,
            searched: HashSet::new(),
            empty: vec![],
            id_map: HashMap::new(),
        }
    }

    fn init(&mut self) {
        // remove all songs from the end that are deleted
        if self.remove_missing {
            self.pop_nonexisting();
        } else {
            self.pop_deleted();
        }

        *self.first_new = self.songs.len();

        self.map_songs();
    }

    fn pop_nonexisting(&mut self) {
        while self
            .songs
            .last()
            .map(|s| {
                if s.is_deleted() {
                    true
                } else if !s.path().exists() {
                    *self.any_removed = true;
                    true
                } else {
                    false
                }
            })
            .unwrap_or(false)
        {
            self.songs.pop();
        }
    }

    fn pop_deleted(&mut self) {
        while self.songs.last().map(|s| s.is_deleted()).unwrap_or(false) {
            self.songs.pop();
        }
    }

    fn map_songs(&mut self) {
        // fill `empty` and `songs`, and remove non existing songs
        for (i, s) in self.songs.iter_mut().enumerate() {
            // sparse deleted song
            if s.is_deleted() {
                self.empty.push(i);
                continue;
            }

            match s.path().canonicalize() {
                // song does exist
                Ok(path) => self.id_map.insert(path, i),
                // song doesn't exist
                Err(_) => {
                    if self.remove_missing {
                        *self.any_removed = true;
                        s.delete();
                        self.empty.push(i);
                    }
                    continue;
                }
            };
        }
    }

    fn load(&mut self) {
        // go trough all the directories
        while let Some(path) = self.paths.pop_first() {
            // open directory
            let dir = err_cont!(
                fs::read_dir(&path),
                err => "Failed to open directory {path:?}: {err}",
            );

            self.searched.insert(path);

            // go trough all the entries in the directory
            for f in dir {
                let file = err_cont!(
                    f,
                    err => "failed to get directory entry: {err}",
                );

                err_cont!(
                    self.add_file(file),
                    err => "failed to examine entry: {err}",
                );
            }
        }
    }

    fn add_file(&mut self, f: DirEntry) -> Result<()> {
        let (ftype, path) = Self::resolve_symlink(f)?;

        // Check directories
        if ftype.is_dir() {
            if self.conf.recursive_search() && !self.searched.contains(&path) {
                self.paths.insert(path);
            }
            return Ok(());
        }

        let Some(fe) = path.extension() else {
            return Ok(());
        };

        // Filter by extension
        if !self
            .conf
            .audio_extensions()
            .iter()
            .any(|e| fe == e.as_str())
        {
            return Ok(());
        }

        // Check if it is already in library.
        if self.id_map.contains_key(&path) {
            return Ok(());
        }

        self.add_song(path)
    }

    fn resolve_symlink(f: DirEntry) -> Result<(FileType, PathBuf)> {
        let ftype = f.file_type()?;

        if ftype.is_symlink() {
            let path = f.path().canonicalize()?;
            Ok((path.metadata()?.file_type(), path))
        } else {
            Ok((ftype, f.path()))
        }
    }

    fn add_song(&mut self, p: PathBuf) -> Result<()> {
        let song = Song::from_path(p)?;

        if let Some(i) = self.empty.pop() {
            self.songs[i] = song;
            self.sparse_new.push(SongId::norm(i));
        } else {
            self.songs.push(song);
        }

        Ok(())
    }
}
