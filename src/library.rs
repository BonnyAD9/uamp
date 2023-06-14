use crate::config::Config;
use crate::song::Song;
use eyre::Result;
use std::fs::read_dir;
use std::ops::Index;

#[derive(Default)]
pub struct Library {
    songs: Vec<Song>,
}

pub enum Filter {
    All,
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

            if let Ok(song) = Song::from_path(f.path()) {
                lib.songs.push(song);
            }

            /*if lib.songs.len() > 100 {
                break;
            }*/
        }

        Ok(lib)
    }

    pub fn filter(&self, filter: Filter) -> Box<dyn Iterator<Item = usize>> {
        match filter {
            Filter::All => Box::new((0..self.songs.len()).into_iter()),
        }
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
