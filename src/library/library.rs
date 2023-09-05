use log::{error, info, warn};
use serde_derive::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use std::{
    cell::Cell,
    fs::{create_dir_all, read_dir, File},
    ops::Index,
    path::Path,
    sync::Arc,
    thread,
    time::Instant,
};

use crate::{
    app::UampApp,
    config::Config,
    core::{
        msg::{ComMsg, Msg},
        Error, Result,
    },
    gen_struct,
};

use super::{
    load::{LibraryLoad, LibraryLoadResult},
    msg::Message,
    Filter, Song, SongId,
};

gen_struct! {
    #[derive(Serialize, Deserialize)]
    pub Library {
        // Fields passed by reference
        songs: Vec<Song> { pri , pri },
        ; // Fields passed by value
        ; // Other fields
        #[serde(skip)]
        load_process: Option<LibraryLoad>,
        /// invalid song
        #[serde(skip, default = "default_ghost")]
        ghost: Song,
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
            load_process: None,
            change: Cell::new(true),
            ghost: Song::invalid(),
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
                    .into_iter()
                    .map(|n| SongId(n))
                    .filter(|s| !self[*s].is_deleted()),
            ),
        }
    }

    /// Saves the library to the default path. Save doesn't happen if
    /// the library didn't change from the last time.
    ///
    /// # Errors
    /// - Fails to create the parent directory
    /// - Fails to write file
    /// - Fails to serialize
    pub fn to_default_json(&self, conf: &Config) -> Result<()> {
        if !self.change.get() {
            return Ok(());
        }
        if let Some(p) = conf.library_path() {
            self.to_json(p)?;
        }
        self.change.set(false);
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

    /// Loads new songs on another thread
    pub fn start_get_new_songs(
        &mut self,
        conf: &Config,
        sender: Arc<UnboundedSender<Msg>>,
    ) -> Result<()> {
        if self.load_process.is_some() {
            return Err(Error::InvalidOperation(
                "Library load is already in progress",
            ));
        }

        let conf = conf.clone();
        let mut songs = self.songs().clone();

        let handle = thread::spawn(move || {
            let conf = conf;
            let songs = if Self::add_new_songs(&mut songs, &conf) {
                Some(songs)
            } else {
                None
            };

            if let Err(e) = sender.send(Msg::Library(Message::LoadEnded)) {
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

impl Default for Library {
    fn default() -> Self {
        Library::new()
    }
}

impl UampApp {
    /// handles library events
    pub fn library_event(&mut self, msg: Message) -> ComMsg {
        match msg {
            Message::LoadEnded => {
                if let Err(e) = self.library.finish_get_new_songs() {
                    error!("Failed to finsih getting new songs: {e}")
                }
                if let Err(e) = self.library.to_default_json(&self.config) {
                    warn!("Failed to save library: {e}");
                }
            }
        }
        ComMsg::none()
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

    /// Finishes the loading of songs started by `start_get_new_songs`
    fn finish_get_new_songs(&mut self) -> Result<()> {
        if let Some(p) = self.load_process.take() {
            let r = p.handle.join().map_err(|_| Error::ThreadPanicked)?;
            if let Some(s) = r.new_song_vec {
                *self.songs_mut() = s;
            }
            Ok(())
        } else {
            Err(Error::InvalidOperation("No load was running"))
        }
    }

    /// Adds new songs to the given vector of songs
    fn add_new_songs(songs: &mut Vec<Song>, conf: &Config) -> bool {
        let mut new_songs = false;
        let mut paths = conf.search_paths().clone();
        let mut i = 0;

        while songs.last().map(|s| s.is_deleted()).unwrap_or(false) {
            songs.pop();
        }

        while i < paths.len() {
            let dir = &paths[i];
            i += 1;

            let dir = match read_dir(dir) {
                Ok(dir) => dir,
                Err(_) => continue,
            };

            'dir_loop: for f in dir {
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
                    if conf.recursive_search() {
                        paths.push(f.path())
                    }
                    continue;
                }

                let path = f.path();

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

                let mut idx = None;

                for i in 0..songs.len() {
                    if songs[i].is_deleted() {
                        // prefer the later indexes, user is more likely to
                        // remove old song and songs at the end are more esily
                        // removed
                        idx = Some(i)
                    }
                    if songs[i].path() == &path {
                        continue 'dir_loop;
                    }
                }

                new_songs = true;

                if let Ok(song) = Song::from_path(path) {
                    if let Some(i) = idx {
                        songs[i] = song;
                    } else {
                        songs.push(song);
                    }
                }
            }
        }

        new_songs
    }
}

fn default_ghost() -> Song {
    Song::invalid()
}
