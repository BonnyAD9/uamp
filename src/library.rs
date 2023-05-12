use crate::config::Config;
use crate::song::Song;
use eyre::Result;
use std::fs::read_dir;

#[derive(Default)]
pub struct Library {
    songs: Vec<Song>,
}

impl Library {
    pub fn new() -> Self {
        Library { songs: Vec::new() }
    }

    pub fn from_config(conf: &Config) -> Result<Self> {
        let mut lib = Library::new();
        let dir = read_dir(conf.library_path())?;

        for f in dir {
            let f = f?;
            let ftype = f.file_type()?;

            if ftype.is_dir() {
                continue;
            }

            let path = f.path();
            let path = match path.to_str() {
                Some(s) => s,
                None => continue,
            };

            if let Ok(song) = Song::from_path(path) {
                lib.songs.push(song);
            }
        }

        Ok(lib)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Song> {
        self.songs.iter()
    }
}
