use crate::config::Config;
use crate::song::Song;
use eyre::Result;
use log::{error, info};
use serde_derive::{Deserialize, Serialize};
use std::fs::{create_dir_all, read_dir, File};
use std::ops::Index;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Library {
    songs: Vec<Song>,
}

#[derive(Debug, Clone, Copy)]
pub struct SongId(usize);

pub enum Filter {
    All,
}

impl Default for Library {
    fn default() -> Self {
        Library::new()
    }
}

impl Library {
    pub fn new() -> Self {
        Library { songs: Vec::new() }
    }

    pub fn from_config(conf: &Config) -> Self {
        Self::from_json(&conf.library_path)
    }

    pub fn filter(&self, filter: Filter) -> Box<dyn Iterator<Item = SongId>> {
        match filter {
            Filter::All => Box::new((0..self.songs.len()).into_iter().map(|n| SongId(n))),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Song> {
        self.songs.iter()
    }

    pub fn to_json(&self, path: impl AsRef<Path>) -> Result<()> {
        if let Some(par) = path.as_ref().parent() {
            create_dir_all(par)?;
        }

        serde_json::to_writer(File::create(path)?, self)?;
        Ok(())
    }

    pub fn from_json(path: impl AsRef<Path>) -> Self {
        if let Ok(file) = File::open(path.as_ref()) {
            serde_json::from_reader(file).unwrap_or_default()
        } else {
            info!("library file {:?} doesn't exist", path.as_ref());
            Self::default()
        }
    }

    pub fn get_new_songs(&mut self, conf: &Config) {
        if Self::add_new_songs(&mut self.songs, conf) {
            if let Err(e) = self.to_json(&conf.library_path) {
                error!(
                    "failed to save library to file {:?}: {e}",
                    conf.library_path
                );
            }
        }
    }

    fn add_new_songs(songs: &mut Vec<Song>, conf: &Config) -> bool {
        let mut new_songs = false;

        for dir in &conf.search_paths {
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
