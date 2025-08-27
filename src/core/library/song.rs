use std::{
    fmt::Debug,
    fs::File,
    path::{Path, PathBuf},
    time::Duration,
};

use audiotags::Tag;
use log::warn;
use raplay::source::{Source, Symph};
use serde::{Deserialize, Serialize};

use crate::core::{
    Error, Result,
    config::{CacheSize, Config},
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Describes song
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    /// Path to the song file.
    path: PathBuf,
    /// Title/Name of the song.
    title: String,
    /// The main artist in the song.
    artist: String,
    /// The album of the song.
    album: String,
    /// The track number in the album.
    track: u32,
    /// The disc number in the album.
    disc: u32,
    /// The year of release.
    year: i32,
    /// The duration/length of the track.
    length: Duration,
    /// The genre of the song.
    genre: String,
    /// True if the song is deleted, deleted songs should be skipped in all
    /// all cases, and should be removed from all collections.
    #[serde(default = "default_deleted")]
    deleted: bool,
}

impl Song {
    /// Creates song from the given path
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let tag = match Tag::new().read_from_path(&path) {
            Ok(tag) => tag,
            Err(audiotags::Error::UnsupportedFormat(_)) => {
                Box::new(audiotags::FlacTag::new())
            }
            Err(audiotags::Error::IOError(e)) => {
                return Error::io(e)
                    .msg(format!(
                        "Failed to read metadata from file `{}`.",
                        path.as_ref().to_string_lossy()
                    ))
                    .err();
            }
            Err(e) => Err(e)?,
        };

        let mut s = Song {
            path: path.as_ref().to_path_buf(),
            title: tag.title().map_or_else(
                || {
                    path.as_ref().file_name().map_or_else(
                        || "--".to_owned(),
                        |a| a.to_string_lossy().into_owned(),
                    )
                },
                |t| t.to_owned(),
            ),
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
            Ok(s.get_time().ok_or(Error::unsupported())?.total)
        };

        match res() {
            Ok(d) => s.length = d,
            Err(e) => warn!(
                "Failed to get true duration of {:?}: {}",
                path.as_ref(),
                e.log()
            ),
        }

        Ok(s)
    }

    pub fn get_cached_path(
        &self,
        conf: &Config,
        size: CacheSize,
    ) -> Option<PathBuf> {
        let mut cached = conf.get_cache_cover_path(size);
        let name = format!("{} - {}.jpg", self.artist, self.album);
        cached.push(filesan::escape_str(&name, '_', filesan::Mode::SYSTEM));

        if cached.exists() { Some(cached) } else { None }
    }

    /// Constructs invalid "ghost" song.
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

    /// Gets the song title/name.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Gets the main artist in the song.
    pub fn artist(&self) -> &str {
        &self.artist
    }

    /// Gets the album of the song.
    pub fn album(&self) -> &str {
        &self.album
    }

    /// Gets the path to the song.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Gets the track number of the song in the album.
    pub fn track(&self) -> u32 {
        self.track
    }

    /// Gets the track number as string.
    pub fn track_str(&self) -> String {
        if self.track == u32::MAX {
            "-".to_owned()
        } else {
            self.track.to_string()
        }
    }

    /// Gets the disc number of the song in the album.
    pub fn disc(&self) -> u32 {
        self.disc
    }

    /// Gets the disc number as string.
    pub fn disc_str(&self) -> String {
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

    /// Marks this song as deleted.
    pub fn delete(&mut self) {
        self.deleted = true;
    }

    /// Gets the year of the release of the song.
    pub fn year(&self) -> i32 {
        self.year
    }

    pub fn year_str(&self) -> String {
        if self.year == i32::MAX {
            "-".to_owned()
        } else {
            self.year.to_string()
        }
    }

    /// Gets the playback length of the song.
    pub fn length(&self) -> Duration {
        self.length
    }

    /// Sets the playback length of the song.
    pub fn set_length(&mut self, len: Duration) {
        self.length = len;
    }

    /// Gets the genre.
    pub fn genre(&self) -> &str {
        &self.genre
    }

    pub fn album_opt(&self) -> Option<&str> {
        (!self.album.is_empty()).then_some(&self.album)
    }

    pub fn artist_opt(&self) -> Option<&str> {
        (!self.artist.is_empty()).then_some(&self.artist)
    }

    pub fn year_str_opt(&self) -> Option<String> {
        (self.year != i32::MAX).then(|| self.year_str())
    }

    pub fn genre_opt(&self) -> Option<&str> {
        (!self.genre.is_empty()).then_some(&self.genre)
    }

    pub fn length_opt(&self) -> Option<Duration> {
        (!self.length.is_zero()).then_some(self.length)
    }

    pub fn title_opt(&self) -> Option<&str> {
        (!self.title.is_empty()).then_some(&self.title)
    }

    pub fn disc_opt(&self) -> Option<u32> {
        (self.disc != u32::MAX).then_some(self.disc)
    }

    pub fn track_opt(&self) -> Option<u32> {
        (self.track != u32::MAX).then_some(self.track)
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

fn default_deleted() -> bool {
    false
}
