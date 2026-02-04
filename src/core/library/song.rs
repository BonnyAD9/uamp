use std::{
    borrow::Cow,
    fmt::Debug,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use itertools::Itertools;
use ratag::{DataType, TagStore, read_tag_from_file, trap};
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        Result,
        config::{CacheSize, Config},
    },
    ext::duration_to_string,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Describes song
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    /// Path to the song file.
    pub(super) path: PathBuf,
    /// Title/Name of the song.
    pub(super) title: Option<String>,
    /// The main artist in the song.
    pub(super) artists: Vec<Arc<str>>,
    /// The album of the song.
    pub(super) album: Option<Arc<str>>,
    /// Artist responsible for the album.
    pub(super) album_artist: Option<Arc<str>>,
    /// The track number in the album.
    pub(super) track: Option<u32>,
    /// The disc number in the album.
    pub(super) disc: Option<u32>,
    /// The year of release.
    pub(super) year: Option<i32>,
    /// The duration/length of the track.
    pub(super) length: Option<Duration>,
    /// The genre of the song.
    pub(super) genres: Vec<String>,
    /// True if the song is deleted, deleted songs should be skipped in all
    /// all cases, and should be removed from all collections.
    #[serde(default = "default_deleted")]
    pub(super) deleted: bool,
}

struct SongTagReader<'a> {
    song: &'a mut Song,
}

impl Song {
    /// Creates song from the given path
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let mut res = Self::empty(path.as_ref());
        SongTagReader::read_to(&mut res, path)?;
        res.artists = res.artists.into_iter().unique().collect();
        res.genres = res.genres.into_iter().unique().collect();
        Ok(res)
    }

    pub fn get_cached_path(
        &self,
        conf: &Config,
        size: CacheSize,
    ) -> Option<PathBuf> {
        let mut cached = conf.get_cache_cover_path(size);
        let name =
            format!("{} - {}.jpg", self.album_artist_str(), self.album_str());
        cached.push(filesan::escape_str(&name, '_', filesan::Mode::SYSTEM));

        if cached.exists() { Some(cached) } else { None }
    }

    /// Constructs invalid "ghost" song.
    pub fn invalid() -> Self {
        Song {
            deleted: true,
            ..Self::empty("<ghost>")
        }
    }

    pub fn empty(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            title: None,
            artists: vec![],
            album: None,
            album_artist: None,
            track: None,
            disc: None,
            year: None,
            length: None,
            genres: vec![],
            deleted: false,
        }
    }

    /// Gets the song title/name.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn title_str(&self) -> &str {
        self.title().unwrap_or("--")
    }

    /// Gets the main artist in the song.
    pub fn artists(&self) -> &[Arc<str>] {
        &self.artists
    }

    pub fn artists_str(&self) -> Cow<'_, str> {
        match self.artists.as_slice() {
            [] => "--".into(),
            [a] => (**a).into(),
            _ => self.artists.iter().join(", ").into(),
        }
    }

    /// Gets the album of the song.
    pub fn album(&self) -> Option<&str> {
        self.album.as_deref()
    }

    pub fn album_str(&self) -> &str {
        self.album().unwrap_or("--")
    }

    pub fn album_artist(&self) -> Option<&str> {
        self.album_artist_arc().map(|a| &**a)
    }

    pub fn album_artist_arc(&self) -> Option<&Arc<str>> {
        self.album_artist
            .as_ref()
            .or_else(|| self.artists().first())
    }

    pub fn album_artist_str(&self) -> &str {
        self.album_artist().unwrap_or("--")
    }

    /// Gets the path to the song.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Gets the track number of the song in the album.
    pub fn track(&self) -> Option<u32> {
        self.track
    }

    /// Gets the track number as string.
    pub fn track_str(&self) -> Cow<'static, str> {
        self.track
            .map(|a| a.to_string().into())
            .unwrap_or("--".into())
    }

    /// Gets the disc number of the song in the album.
    pub fn disc(&self) -> Option<u32> {
        self.disc
    }

    /// Gets the disc number as string.
    pub fn disc_str(&self) -> Cow<'static, str> {
        self.disc
            .map(|a| a.to_string().into())
            .unwrap_or("--".into())
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
    pub fn year(&self) -> Option<i32> {
        self.year
    }

    pub fn year_str(&self) -> Cow<'static, str> {
        self.year
            .map(|a| a.to_string().into())
            .unwrap_or("--".into())
    }

    /// Gets the playback length of the song.
    pub fn length(&self) -> Option<Duration> {
        self.length
    }

    pub fn length_str(&self, truncate: bool) -> Cow<'static, str> {
        self.length
            .map(|a| duration_to_string(a, truncate).into())
            .unwrap_or("--".into())
    }

    /// Sets the playback length of the song.
    pub fn set_length(&mut self, len: Duration) {
        self.length = Some(len);
    }

    /// Gets the genre.
    pub fn genres(&self) -> &[String] {
        &self.genres
    }

    pub fn genres_str(&self) -> Cow<'_, str> {
        match self.genres.as_slice() {
            [] => "--".into(),
            [a] => a.as_str().into(),
            _ => self.artists.iter().join(", ").into(),
        }
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl<'a> SongTagReader<'a> {
    pub fn new(s: &'a mut Song) -> Self {
        Self { song: s }
    }

    pub fn read_to(s: &'a mut Song, p: impl AsRef<Path>) -> Result<()> {
        let mut r = Self::new(s);
        match read_tag_from_file(p, &mut r, &trap::Skip) {
            Err(ratag::Error::NoTag) => Ok(()),
            r => r.map_err(|e| e.into()),
        }
    }
}

impl<'a> TagStore for SongTagReader<'a> {
    fn stores_data(&self, typ: DataType) -> bool {
        matches!(
            typ,
            DataType::Title
                | DataType::Artists
                | DataType::Album
                | DataType::AlbumArtist
                | DataType::Track
                | DataType::Disc
                | DataType::Year
                | DataType::Length
                | DataType::Genres
        )
    }

    fn set_title(&mut self, title: String) {
        if !title.is_empty() {
            self.song.title = Some(title);
        }
    }

    fn set_artists(&mut self, artists: Vec<String>) {
        if !artists.is_empty() {
            self.song.artists =
                artists.iter().map(|a| a.as_str().into()).collect();
        }
    }

    fn set_album(&mut self, album: String) {
        if !album.is_empty() {
            self.song.album = Some(album.as_str().into());
        }
    }

    fn set_album_artist(&mut self, artist: String) {
        if !artist.is_empty() {
            self.song.album_artist = Some(artist.as_str().into());
        }
    }

    fn set_track(&mut self, track: u32) {
        if track != 0 {
            self.song.track = Some(track);
        }
    }

    fn set_disc(&mut self, disc: u32) {
        if disc != 0 {
            self.song.disc = Some(disc);
        }
    }

    fn set_year(&mut self, year: i32) {
        self.song.year = Some(year);
    }

    fn set_length(&mut self, length: Duration) {
        self.song.length = Some(length);
    }

    fn set_genres(&mut self, genres: Vec<String>) {
        if !genres.is_empty() {
            self.song.genres = genres;
        }
    }
}

fn default_deleted() -> bool {
    false
}
