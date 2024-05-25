use std::{collections::{BTreeSet, HashMap, HashSet}, fs};

use log::error;

use crate::config::Config;

use super::{LibraryLoadResult, Song, SongId};

/// Adds new songs to the given vector of songs
pub(super) fn add_new_songs(res: &mut LibraryLoadResult, conf: &Config, remove_missing: bool) -> bool {
    let mut new_songs = false;
    // directories that were already searched
    let mut searched = HashSet::new();
    // directories to be searched
    let mut paths: BTreeSet<_> = conf.search_paths().iter().flat_map(|p| p.canonicalize()).collect();

    let mut remove_any = false;

    // remove all songs from the end that are deleted
    if remove_missing {
        while res
            .songs
            .last()
            .map(|s| {
                if s.is_deleted() {
                    true
                } else if !s.path().exists() {
                    remove_any = true;
                    true
                } else {
                    false
                }
            })
            .unwrap_or(false)
        {
            res.songs.pop();
        }
    } else {
        while res.songs.last().map(|s| s.is_deleted()).unwrap_or(false) {
            res.songs.pop();
        }
    }

    res.first_new = res.songs.len();

    // indexes of sparse deleted songs
    let mut empty = vec![];
    // maps canonicalized paths to song ids
    let mut songs = HashMap::new();

    // fill `empty` and `songs`, and remove non existing songs
    for (i, s) in res.songs.iter_mut().enumerate() {
        // sparse deleted song
        if s.is_deleted() {
            empty.push(i);
            continue;
        }

        let path = match s.path().canonicalize() {
            // song does exist
            Ok(path) => path,
            // song doesn't exist
            Err(_) => {
                if remove_missing {
                    remove_any = true;
                    s.delete();
                }
                empty.push(i);
                continue;
            }
        };

        songs.insert(path, i);
    }

    res.removed = remove_any;
    res.first_new = res.songs.len();

    // go trough all the directories
    while let Some(path) = paths.pop_first() {
        // open directory
        let dir = match fs::read_dir(&path) {
            Ok(dir) => dir,
            Err(e) => {
                error!("Failed to open directory {path:?}: {e}");
                continue;
            },
        };

        searched.insert(path);

        // go trough all the entries in the directory
        for f in dir {
            let f = match f {
                Ok(f) => f,
                Err(e) => {
                    error!("failed to get directory entry: {e}");
                    continue;
                }
            };

            let ftype = match f.file_type() {
                Ok(ft) => ft,
                Err(e) => {
                    error!(
                        "failed to get directory entry type of {f:?}: {e}"
                    );
                    continue;
                }
            };

            // properly resolve symlinks
            let (ftype, path) = if ftype.is_symlink() {
                let path = f.path();
                let path = match path.canonicalize() {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Failed to canonicalize the path {path:?}: {e}");
                        continue;
                    }
                };
                let ftype = match path.metadata() {
                    Ok(m) => m.file_type(),
                    Err(e) => {
                        error!("Failed to get metadata for the file {path:?}: {e}");
                        continue;
                    }
                };
                (ftype, path)
            } else {
                (ftype, f.path())
            };

            if ftype.is_dir() {
                if conf.recursive_search() && !searched.contains(&path) {
                    paths.insert(path);
                }
                continue;
            }

            if let Some(fe) = path.extension() {
                if !conf
                    .audio_extensions()
                    .iter()
                    .any(|e| fe == e.as_str())
                {
                    continue;
                }
            } else {
                continue;
            }

            if songs.contains_key(&path) {
                continue;
            }

            // add new song to the library
            new_songs = true;

            let song = match Song::from_path(&path) {
                Ok(song) => song,
                Err(e) => {
                    error!("Failed to add song {path:?}: {e}");
                    continue;
                }
            };

            if let Some(i) = empty.pop() {
                res.songs[i] = song;
                res.sparse_new.push(SongId(i));
            } else {
                res.songs.push(song);
            }
        }
    }

    new_songs
}
