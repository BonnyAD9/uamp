use crate::config::Config;
use crate::song::Song;
use eyre::Result;
use std::fs::read_dir;
use std::ops::Index;

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

            if let Ok(song) = Song::from_path(path.to_owned()) {
                lib.songs.push(song);
            }
        }

        // songs are identified by their id, which is index in the song vector
        // so the library is sorted to make related songs close to each other
        lib.songs.sort_unstable_by_key(|p| p.track());
        lib.songs.sort_by_key(|p| p.disc());
        lib.songs.sort_by(|a, b| a.album().cmp(b.album()));
        lib.songs.sort_by(|a, b| a.artist().cmp(b.artist()));

        Ok(lib)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Song> {
        self.songs.iter()
    }
}

impl Index<usize> for Library {
    type Output = Song;
    fn index(&self, index: usize) -> &Self::Output {
        &self.songs[index]
    }
}
