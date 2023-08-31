use crate::err::Error;
use crate::uamp_app::UampMessage;
use crate::wid::Command;
use crate::{config::Config, err::Result, song::Song};
use log::{error, info, warn};
use serde_derive::{Deserialize, Serialize};
use std::fs::{create_dir_all, read_dir, File};
use std::ops::Index;
use std::path::Path;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedSender;

/// A song library
#[derive(Serialize, Deserialize)]
pub struct Library {
    songs: Vec<Song>,
    #[serde(skip)]
    load_process: Option<LibraryLoad>,
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
        Library {
            songs: Vec::new(),
            load_process: None,
        }
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

    /// Loads new songs on another thread
    pub fn start_get_new_songs(
        &mut self,
        conf: &Config,
        sender: Arc<UnboundedSender<UampMessage>>,
    ) -> Result<()> {
        if self.load_process.is_some() {
            return Err(Error::InvalidOperation(
                "Library load is already in progress",
            ));
        }

        let conf = conf.clone();
        let mut songs = self.songs.clone();

        let handle = thread::spawn(move || {
            let conf = conf;
            let songs = if Self::add_new_songs(&mut songs, &conf) {
                Some(songs)
            } else {
                None
            };

            if let Err(e) =
                sender.send(UampMessage::Library(LibraryMessage::LoadEnded))
            {
                error!("Library load failed to send message: {e}");
            }

            LibraryLoadResult {
                new_song_vec: songs,
            }
        });

        self.load_process = Some(LibraryLoad {
            handle,
            time_started: Instant::now(),
        });

        Ok(())
    }

    fn finish_get_new_songs(&mut self) -> Result<()> {
        if let Some(p) = self.load_process.take() {
            let r = p.handle.join().map_err(|_| Error::ThreadPanicked)?;
            if let Some(s) = r.new_song_vec {
                self.songs = s;
            }
            Ok(())
        } else {
            Err(Error::InvalidOperation("No load was running"))
        }
    }

    pub fn event(&mut self, msg: LibraryMessage, config: &Config) -> Command {
        match msg {
            LibraryMessage::LoadEnded => {
                if let Err(e) = self.finish_get_new_songs() {
                    error!("Failed to finsih getting new songs: {e}")
                }
                if let Err(e) = self.to_json(&config.library_path) {
                    warn!("Failed to save library: {e}");
                }
            }
        }
        Command::none()
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

struct LibraryLoad {
    handle: JoinHandle<LibraryLoadResult>,
    time_started: Instant,
}

struct LibraryLoadResult {
    new_song_vec: Option<Vec<Song>>,
}

#[derive(Clone, Debug)]
pub enum LibraryMessage {
    LoadEnded,
}
