use std::{
    fmt::Debug,
    fs::File,
    path::{Path, PathBuf},
    time::Duration,
};

use audiotags::Tag;
use log::warn;
use raplay::source::{Source, Symph};
use serde_derive::{Deserialize, Serialize};

use crate::core::{extensions::duration_to_string, Error, Result};

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
    /// The year of release
    year: i32,
    /// The duration/length of the track
    length: Duration,
    /// The genre of the song
    genre: String,
    /// True if the song is deleted, deleted songs should be skipped in all
    /// all cases, and should be removed from all collections
    #[serde(default = "default_deleted")]
    deleted: bool,
}

impl Song {
    /// Creates song from the given path
    pub fn from_path<P: AsRef<Path> + Debug>(path: P) -> Result<Self> {
        let tag = Tag::new().read_from_path(&path)?;
        let mut s = Song {
            path: path.as_ref().to_path_buf(),
            title: tag.title().unwrap_or("-").to_owned(),
            artist: tag.artist().unwrap_or("-").to_owned(),
            album: tag.album_title().unwrap_or("-").to_owned(),
            track: tag.track().0.unwrap_or_default() as u32,
            disc: tag.disc().0.unwrap_or_default() as u32,
            year: tag.year().unwrap_or(i32::MAX),
            length: tag
                .duration()
                .map(Duration::from_secs_f64)
                .unwrap_or(Duration::ZERO),
            genre: tag.genre().unwrap_or("-").to_owned(),
            deleted: false,
        };

        let res = || -> Result<Duration> {
            let f = File::open(&path)?;
            let s = Symph::try_new(f, &Default::default())?;
            Ok(s.get_time()
                .ok_or(Error::InvalidOperation("Not supported"))?
                .total)
        };

        match res() {
            Ok(d) => s.length = d,
            Err(e) => warn!("Failed to get true duration of {:?}: {e}", path),
        }

        Ok(s)
    }

    /// Constructs invalid "ghost" song
    pub fn invalid() -> Self {
        Self {
            path: "<ghost>".into(),
            title: "<ghost>".to_owned(),
            artist: "<ghost>".to_owned(),
            album: "<ghost>".to_owned(),
            track: u32::MAX,
            disc: u32::MAX,
            year: i32::MAX,
            length: Duration::ZERO,
            genre: "<ghost>".to_owned(),
            deleted: true,
        }
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

    pub fn _track_str(&self) -> String {
        if self.track == u32::MAX {
            "-".to_owned()
        } else {
            self.track.to_string()
        }
    }

    /// Gets the disc number of the song in the album
    pub fn disc(&self) -> u32 {
        self.disc
    }

    pub fn _disc_str(&self) -> String {
        if self.disc == u32::MAX {
            "-".to_owned()
        } else {
            self.disc.to_string()
        }
    }

    /// Returns true if the song is deleted
    pub fn is_deleted(&self) -> bool {
        self.deleted
    }

    pub fn delete(&mut self) {
        self.deleted = true;
    }

    pub fn _year(&self) -> i32 {
        self.year
    }

    pub fn _year_str(&self) -> String {
        if self.year == i32::MAX {
            "-".to_owned()
        } else {
            self.year.to_string()
        }
    }

    pub fn length(&self) -> Duration {
        self.length
    }

    pub fn _length_str(&self) -> String {
        if self.length == Duration::ZERO {
            "--:--".to_owned()
        } else {
            duration_to_string(self.length, true)
        }
    }

    pub fn set_length(&mut self, len: Duration) {
        self.length = len;
    }

    pub fn _genre(&self) -> &str {
        &self.genre
    }
}

fn default_deleted() -> bool {
    false
}
