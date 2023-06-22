use std::path::{Path, PathBuf};

use audiotags::Tag;
use eyre::Result;
use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    path: PathBuf,
    title: String,
    artist: String,
    album: String,
    track: u32,
    disc: u32,
}

impl Song {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let tag = Tag::new().read_from_path(&path)?;
        Ok(Song {
            path: path.as_ref().to_path_buf(),
            title: tag.title().unwrap_or("<unknown title>").to_owned(),
            artist: tag.artist().unwrap_or("<unknown artist>").to_owned(),
            album: tag.album_title().unwrap_or("<unknown album>").to_owned(),
            track: tag.track().0.unwrap_or_default() as u32,
            disc: tag.disc().0.unwrap_or_default() as u32,
        })
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn artist(&self) -> &str {
        &self.artist
    }

    pub fn album(&self) -> &str {
        &self.album
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn track(&self) -> u32 {
        self.track
    }

    pub fn disc(&self) -> u32 {
        self.disc
    }
}
