use crate::config::Config;
use crate::song::Song;
use eyre::Result;
use log::{error, info};
use serde_derive::{Deserialize, Serialize};
use std::fs::{create_dir_all, read_dir, File};
use std::ops::Index;
use std::path::Path;

/// A song library
#[derive(Serialize, Deserialize)]
pub struct Library {
    songs: Vec<Song>,
}

/// Id of song in a [`Library`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SongId(usize);

/// Filter for iterating library
pub enum Filter {
    All,
}

impl Default for Library {
    fn default() -> Self {
        Library::new()
    }
}

impl Library {
    /// Creates empty library
    pub fn new() -> Self {
        Library { songs: Vec::new() }
    }

    /// Loads library according to config, returns empty library on fail
    pub fn from_config(conf: &Config) -> Self {
        Self::from_json(&conf.library_path)
    }

    /// Filters songs in the library
    pub fn filter(&self, filter: Filter) -> Box<dyn Iterator<Item = SongId>> {
        match filter {
            Filter::All => {
                Box::new((0..self.songs.len()).into_iter().map(|n| SongId(n)))
            }
        }
    }

    /// Iterates over all of the songs in the library
    pub fn iter(&self) -> std::slice::Iter<'_, Song> {
        self.songs.iter()
    }

    /// Saves the library to the specified path.
    ///
    /// # Errors
    /// - Fails to create the parent directory
    /// - Fails to write file
    /// - Fails to serialize
    pub fn to_json(&self, path: impl AsRef<Path>) -> Result<()> {
        if let Some(par) = path.as_ref().parent() {
            create_dir_all(par)?;
        }

        serde_json::to_writer(File::create(path)?, self)?;
        Ok(())
    }

    /// Loads the library from the given json file. Returns default library on
    /// error.
    pub fn from_json(path: impl AsRef<Path>) -> Self {
        if let Ok(file) = File::open(path.as_ref()) {
            serde_json::from_reader(file).unwrap_or_default()
        } else {
            info!("library file {:?} doesn't exist", path.as_ref());
            Self::default()
        }
    }

    /// Finds new songs to the library
    pub fn get_new_songs(&mut self, conf: &Config) {
        Self::add_new_songs(&mut self.songs, conf);
    }

    /// Adds new songs to the given vector of songs
    fn add_new_songs(songs: &mut Vec<Song>, conf: &Config) -> bool {
        let mut new_songs = false;
        let mut paths = conf.search_paths.clone();
        let mut i = 0;

        while i < paths.len() {
            let dir = &paths[i];
            i += 1;

            let dir = match read_dir(dir) {
                Ok(dir) => dir,
                Err(_) => continue,
            };

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

                if ftype.is_dir() {
                    if conf.recursive_search {
                        paths.push(f.path())
                    }
                    continue;
                }

                let path = f.path();

                if let Some(fe) = path.extension() {
                    if !conf.audio_extensions.iter().any(|e| fe == e.as_str())
                    {
                        continue;
                    }
                } else {
                    continue;
                }

                if songs.iter().any(|s| *s.path() == path) {
                    continue;
                }

                new_songs = true;

                if let Ok(song) = Song::from_path(path) {
                    songs.push(song);
                }
            }
        }

        new_songs
    }
}

impl Index<SongId> for Library {
    type Output = Song;
    fn index(&self, index: SongId) -> &Self::Output {
        &self.songs[index.0]
    }
}
