use std::path::{Path, PathBuf};

use audiotags::Tag;
use eyre::Result;
use serde_derive::{Deserialize, Serialize};

/// Describes song
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    /// Path to the song file
    path: PathBuf,
    /// Title/Name of the song
    title: String,
    /// The main artist in the song
    artist: String,
    /// The album of the song
    album: String,
    /// The track number in the album
    track: u32,
    /// The disc number in the album
    disc: u32,
}

impl Song {
    /// Creates song from the given path
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

    /// Gets the song title/name
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Gets the main artist in the song
    pub fn artist(&self) -> &str {
        &self.artist
    }

    /// Gets the album of the song
    pub fn album(&self) -> &str {
        &self.album
    }

    /// Gets the path to the song
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Gets the track number of the song in the album
    pub fn track(&self) -> u32 {
        self.track
    }

    /// Gets the disc number of the song in the album
    pub fn disc(&self) -> u32 {
        self.disc
    }
}
