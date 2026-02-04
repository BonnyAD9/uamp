use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs::{self, DirEntry, FileType},
    mem,
    path::PathBuf,
    sync::Arc,
};

use log::error;

use crate::core::{
    Result,
    config::Config,
    library::{Album, Albums, Artist, Artists},
};

use super::{LibraryLoadResult, Song, SongId};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Adds new songs to the given vector of songs
pub(super) fn add_new_songs(
    res: &mut LibraryLoadResult,
    conf: &Config,
    remove_missing: bool,
) {
    let mut state = State::new(res, conf, remove_missing);
    state.init();
    state.load();
    state.finalize();
}

pub(super) fn construct_album_artists(
    songs: &mut [Song],
) -> (Albums, Artists) {
    let mut albums = HashMap::new();
    let mut artists = HashMap::new();

    for (id, song) in songs.iter_mut().enumerate() {
        add_song_album_artists(
            song,
            SongId::norm(id),
            &mut albums,
            &mut artists,
        );
    }

    for album in albums.values_mut() {
        normalize_album(album, songs);
    }

    for artist in artists.values_mut() {
        normalize_artist(artist, songs);
    }

    (albums, artists)
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

struct State<'a> {
    conf: &'a Config,
    remove_missing: bool,
    // All the songs in the library, this is the main result.
    songs: &'a mut Vec<Song>,
    // All the albums in the library.
    albums: &'a mut Albums,
    // All the artists in the library.
    artists: &'a mut Artists,
    // Ids of new songs that are not at the end of `songs`.
    sparse_new: &'a mut Vec<SongId>,
    // Index of the first song that is new at the end of `songs`.
    first_new: &'a mut usize,
    // True if some songs were removed from `songs`.
    any_removed: &'a mut bool,
    // Directories to visit.
    paths: BTreeSet<PathBuf>,
    // Already visited directories.
    searched: HashSet<PathBuf>,
    // Indexes of all deleted songs in `songs`.
    empty: Vec<usize>,
    // Maps song path to its id.
    id_map: HashMap<PathBuf, usize>,
    // Names of modified albums that should be normalized.
    modified_albums: BTreeSet<(Arc<str>, Arc<str>)>,
    // Names of modifed artists that should be normalized.
    modified_artists: BTreeSet<Arc<str>>,
}

macro_rules! err_cont {
    ($e:expr, $err:ident => $msg:literal $(,)?) => {
        match $e {
            Ok(v) => v,
            Err($err) => {
                error!($msg);
                continue;
            }
        }
    };
}

impl<'a> State<'a> {
    fn new(
        res: &'a mut LibraryLoadResult,
        conf: &'a Config,
        remove_missing: bool,
    ) -> Self {
        let paths = conf
            .search_paths()
            .iter()
            .flat_map(|p| p.canonicalize())
            .collect();

        Self {
            conf,
            remove_missing,
            songs: &mut res.songs,
            albums: &mut res.albums,
            artists: &mut res.artists,
            sparse_new: &mut res.sparse_new,
            first_new: &mut res.first_new,
            any_removed: &mut res.removed,
            paths,
            searched: HashSet::new(),
            empty: vec![],
            id_map: HashMap::new(),
            modified_albums: BTreeSet::new(),
            modified_artists: BTreeSet::new(),
        }
    }

    fn init(&mut self) {
        let mut removed = vec![];

        // remove all songs from the end that are deleted
        if self.remove_missing {
            self.pop_nonexisting(&mut removed);
        } else {
            self.pop_deleted();
        }

        *self.first_new = self.songs.len();

        self.map_songs(&mut removed);

        *self.any_removed = !removed.is_empty();
        self.propagate_remove(removed);
    }

    /// pops all invalid songs (deleted == true) and songs that don't exist
    /// (path doesn't exist).
    fn pop_nonexisting(&mut self, removed: &mut Vec<(SongId, Song)>) {
        while let Some(s) = self.songs.last() {
            if s.is_deleted() {
                self.songs.pop();
                continue;
            }

            if s.path().exists() {
                break;
            }

            let s = self.songs.pop().unwrap();

            removed.push((SongId::norm(self.songs.len()), s));
        }
    }

    /// pops all invalid songs (have deleted == true)
    fn pop_deleted(&mut self) {
        while self.songs.last().map(|s| s.is_deleted()).unwrap_or(false) {
            self.songs.pop();
        }
    }

    fn map_songs(&mut self, removed: &mut Vec<(SongId, Song)>) {
        // fill `empty` and `songs`, and remove non existing songs
        for (i, s) in self.songs.iter_mut().enumerate() {
            // sparse deleted song
            if s.is_deleted() {
                self.empty.push(i);
                continue;
            }

            match s.path().canonicalize() {
                // song does exist
                Ok(path) => self.id_map.insert(path, i),
                // song doesn't exist
                Err(_) => {
                    if self.remove_missing {
                        *self.any_removed = true;
                        removed.push((
                            SongId::norm(i),
                            mem::replace(s, Song::invalid()),
                        ));
                        self.empty.push(i);
                    }
                    continue;
                }
            };
        }
    }

    fn propagate_remove(&mut self, removed: Vec<(SongId, Song)>) {
        let mut rem_alb: BTreeMap<(Arc<str>, Arc<str>), Vec<SongId>> =
            BTreeMap::new();
        let mut rem_singles: BTreeMap<Arc<str>, Vec<SongId>> = BTreeMap::new();
        for (id, mut s) in removed {
            let Some(album) = s.album else {
                for a in s.artists {
                    rem_singles.entry(a).or_default().push(id);
                }
                continue;
            };

            if s.album_artist.is_none() && !s.artists.is_empty() {
                s.album_artist = Some(s.artists.swap_remove(0));
            }

            if let Some(aa) = &s.album_artist {
                s.artists.retain(|a| a != aa);
            }

            let Some(album_artist) = s.album_artist else {
                continue;
            };

            rem_alb.entry((album_artist, album)).or_default().push(id);
            for a in s.artists {
                rem_singles.entry(a).or_default().push(id);
            }
        }

        let rem_art = self.remove_from_albums(rem_alb);
        self.remove_from_singles(rem_singles);
        self.remove_from_artists(rem_art);
    }

    fn remove_from_albums(
        &mut self,
        rem_alb: impl IntoIterator<Item = ((Arc<str>, Arc<str>), Vec<SongId>)>,
    ) -> BTreeMap<Arc<str>, Vec<Arc<str>>> {
        let mut rem_art: BTreeMap<Arc<str>, Vec<Arc<str>>> = BTreeMap::new();
        for (key, mut songs) in rem_alb {
            let Some(alb) = self.albums.get_mut(&key) else {
                continue;
            };

            alb.songs.retain(|a| {
                if let Some(idx) = songs.iter().position(|i| i == a) {
                    songs.swap_remove(idx);
                    false
                } else {
                    true
                }
            });

            if alb.songs.is_empty() {
                let alb = self.albums.remove(&key).unwrap();
                rem_art.entry(alb.artist).or_default().push(alb.name);
            }
        }

        rem_art
    }

    fn remove_from_singles(
        &mut self,
        rem_singles: impl IntoIterator<Item = (Arc<str>, Vec<SongId>)>,
    ) {
        for (key, mut songs) in rem_singles {
            let Some(art) = self.artists.get_mut(&key) else {
                continue;
            };

            art.singles.retain(|a| {
                if let Some(idx) = songs.iter().position(|i| i == a) {
                    songs.swap_remove(idx);
                    false
                } else {
                    true
                }
            });

            if art.singles.is_empty() && art.albums.is_empty() {
                self.artists.remove(&key);
            }
        }
    }

    fn remove_from_artists(
        &mut self,
        rem_art: impl IntoIterator<Item = (Arc<str>, Vec<Arc<str>>)>,
    ) {
        for (key, mut albums) in rem_art {
            let Some(art) = self.artists.get_mut(&key) else {
                continue;
            };

            art.albums.retain(|a| {
                if let Some(idx) = albums.iter().position(|i| i == a) {
                    albums.swap_remove(idx);
                    false
                } else {
                    true
                }
            });

            if art.singles.is_empty() && art.albums.is_empty() {
                self.artists.remove(&key);
            }
        }
    }

    fn load(&mut self) {
        // go trough all the directories
        while let Some(path) = self.paths.pop_first() {
            // open directory
            let dir = err_cont!(
                fs::read_dir(&path),
                err => "Failed to open directory {path:?}: {err}",
            );

            self.searched.insert(path);

            // go trough all the entries in the directory
            for f in dir {
                let file = err_cont!(
                    f,
                    err => "failed to get directory entry: {err}",
                );

                err_cont!(
                    self.add_file(file),
                    err => "failed to examine entry: {err}",
                );
            }
        }
    }

    /// Add file/subdirectory
    fn add_file(&mut self, f: DirEntry) -> Result<()> {
        let (ftype, path) = Self::resolve_symlink(f)?;

        // Check directories
        if ftype.is_dir() {
            if self.conf.recursive_search() && !self.searched.contains(&path) {
                self.paths.insert(path);
            }
            return Ok(());
        }

        let Some(fe) = path.extension() else {
            return Ok(());
        };

        // Filter by extension
        if !self
            .conf
            .audio_extensions()
            .iter()
            .any(|e| fe == e.as_str())
        {
            return Ok(());
        }

        // Check if it is already in library.
        if self.id_map.contains_key(&path) {
            return Ok(());
        }

        self.add_song(path)
    }

    /// Gets the file type and canonical path.
    fn resolve_symlink(f: DirEntry) -> Result<(FileType, PathBuf)> {
        let ftype = f.file_type()?;

        if ftype.is_symlink() {
            let path = f.path().canonicalize()?;
            Ok((path.metadata()?.file_type(), path))
        } else {
            Ok((ftype, f.path()))
        }
    }

    fn add_song(&mut self, p: PathBuf) -> Result<()> {
        let song = Song::from_path(p)?;

        // Assign id to the song by inserting it to the library.
        let id = if let Some(i) = self.empty.pop() {
            self.songs[i] = song;
            let id = SongId::norm(i);
            self.sparse_new.push(id);
            id
        } else {
            let id = SongId::norm(self.songs.len());
            self.songs.push(song);
            id
        };

        let song = &mut self.songs[id.as_norm()];

        add_song_album_artists(song, id, self.albums, self.artists);

        Ok(())
    }

    fn finalize(&mut self) {
        self.normalize_albums();
        self.normalize_artists();
    }

    fn normalize_albums(&mut self) {
        for album in &self.modified_albums {
            if let Some(album) = self.albums.get_mut(album) {
                normalize_album(album, self.songs);
            }
        }
    }

    fn normalize_artists(&mut self) {
        for artist in &self.modified_artists {
            if let Some(artist) = self.artists.get_mut(artist) {
                normalize_artist(artist, self.songs);
            }
        }
    }
}

fn add_song_album_artists(
    song: &mut Song,
    id: SongId,
    albums: &mut Albums,
    artists: &mut Artists,
) {
    let Some(album_artist) = song.album_artist_arc() else {
        // Non existance of album artist implies that artists is empty.
        return;
    };

    let album_artist = artists
        .entry(album_artist.clone())
        .or_insert_with(|| Artist::new(album_artist.clone()));
    if song.album_artist.is_some() {
        // Reduce number of copies in memory.
        song.album_artist = Some(album_artist.name.clone());
    }

    // Create/asociate with album
    if let Some(album) = &song.album {
        let album = albums
            .entry((album_artist.name.clone(), album.clone()))
            .or_insert_with_key(|(art, nam)| {
                album_artist.albums.push(nam.clone());
                Album::new(art.clone(), nam.clone())
            });
        // Reduce number of copies in memory.
        song.album = Some(album.name.clone());
        album.songs.push(id);
    } else {
        album_artist.singles.push(id);
    }

    let aa = album_artist.name.clone();
    for sart in &mut song.artists {
        let artist = artists
            .entry(sart.clone())
            .or_insert_with_key(|k| Artist::new(k.clone()));
        *sart = artist.name.clone();

        if artist.name != aa {
            artist.singles.push(id);
        }
    }
}

fn normalize_album(album: &mut Album, songs: &[Song]) {
    album.songs.sort_by_key(|id| songs[id.as_norm()].track());
}

fn normalize_artist(artist: &mut Artist, songs: &[Song]) {
    artist.albums.sort();
    artist.singles.sort_by_key(|id| songs[id.as_norm()].title());
}
