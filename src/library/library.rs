use audiotags::Tag;
use iced_core::image::{Data, Handle};
use itertools::Itertools;
use log::{error, info, warn};
use serde_derive::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use std::{
    borrow::Cow,
    cell::Cell,
    collections::HashMap,
    fs::{create_dir_all, read_dir, File},
    io::{Read, Write},
    mem,
    ops::{Index, IndexMut},
    path::Path,
    sync::Arc,
    thread::{self, JoinHandle},
    time::Instant,
};

use crate::{
    app::UampApp,
    config::Config,
    core::{
        extensions::valid_filename,
        msg::{ComMsg, Msg},
        Error, Result,
    },
    gen_struct,
};

use super::{
    cover_image::CoverImage,
    load::{LibraryLoad, LibraryLoadResult},
    msg::Message,
    Filter, LibraryUpdate, Song, SongId,
};

type ImageMap = HashMap<(Cow<'static, str>, Cow<'static, str>), CoverImage>;

gen_struct! {
    #[derive(Serialize, Deserialize)]
    pub Library {
        // Fields passed by reference
        songs: Vec<Song> { pri , pri },
        // albums: Vec<SongId> { pri, pri },
        ; // Fields passed by value
        ; // Other fields
        #[serde(skip)]
        load_process: Option<LibraryLoad>,
        #[serde(skip)]
        save_process: Option<JoinHandle<Result<()>>>,
        #[serde(skip)]
        image_load_process: Option<JoinHandle<Library>>,
        #[serde(skip)]
        image_shrink_process: Option<JoinHandle<ImageMap>>,
        #[serde(skip)]
        new_images: bool,
        /// invalid song
        #[serde(skip, default = "default_ghost")]
        ghost: Song,
        #[serde(skip)]
        lib_update: LibraryUpdate,
        #[serde(skip)]
        images: ImageMap,
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
            save_process: None,
            image_load_process: None,
            image_shrink_process: None,
            new_images: false,
            lib_update: LibraryUpdate::None,
            change: Cell::new(true),
            ghost: Song::invalid(),
            images: Default::default(),
        }
    }

    pub fn update(&mut self, up: LibraryUpdate) {
        if up > self.lib_update {
            self.lib_update = up;
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

    pub fn start_load_images(
        &mut self,
        sender: Arc<UnboundedSender<Msg>>,
        conf: &Config,
    ) -> Result<()> {
        if self.image_load_process.is_some() {
            return Err(Error::InvalidOperation(
                "cannot load images, load is already in process",
            ));
        }

        let conf = conf.clone();
        let mut lib = self.clone();

        self.image_load_process = Some(thread::spawn(move || {
            lib.load_images(&conf);
            if let Err(e) = sender.send(Msg::Library(Message::ImageLoadEnded))
            {
                error!("Failed to send image load end message: {e}");
            }
            lib
        }));

        Ok(())
    }

    pub fn start_make_thumbnails(
        &mut self,
        sender: Arc<UnboundedSender<Msg>>,
        conf: &Config,
    ) -> Result<()> {
        if self.image_shrink_process.is_some() {
            return Err(Error::InvalidOperation(
                "Cannot make thumbnails, the process is already in progress",
            ));
        }

        let conf = conf.clone();
        let mut imgs = self.images.clone();

        self.image_shrink_process = Some(thread::spawn(move || {
            if let Err(e) = Self::make_thumbnails(&mut imgs, &conf) {
                error!("Failed to make thumbnails: {e}");
            }
            if let Err(e) =
                sender.send(Msg::Library(Message::ImageShrinkEnded))
            {
                error!("Failed to send image shrink end message: {e}");
            }
            imgs
        }));

        Ok(())
    }

    pub fn get_image(&self, s: SongId) -> Option<Handle> {
        let s = &self[s];
        self.images
            .get(&(s.artist().into(), s.album().into()))
            .map(|v| v.as_andle())
    }

    pub fn get_small_image(&self, s: SongId) -> Option<Handle> {
        let s = &self[s];
        self.images
            .get(&(s.artist().into(), s.album().into()))
            .and_then(|v| v.as_small())
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

    pub fn start_to_default_json(
        &mut self,
        conf: &Config,
        sender: Arc<UnboundedSender<Msg>>,
    ) -> Result<()> {
        if !self.change.get() {
            return Ok(());
        }

        // End panicked processes
        self.any_process();

        if self.save_process.is_some() {
            return Err(Error::InvalidOperation(
                "Library load is already in progress",
            ));
        }

        if let Some(p) = conf.library_path() {
            let path = p.clone();
            let me = self.clone();

            let handle = thread::spawn(move || -> Result<()> {
                let path = path;
                me.to_json(path)?;
                if let Err(e) = sender.send(Msg::Library(Message::SaveEnded)) {
                    error!("Library save failed to send message: {e}");
                }
                Ok(())
            });

            self.save_process = Some(handle);
        }

        self.change.set(false);

        Ok(())
    }

    /// Loads the library from the given json file. Returns default library on
    /// error.
    pub fn from_json(path: impl AsRef<Path>) -> Self {
        if let Ok(file) = File::open(path.as_ref()) {
            match serde_json::from_reader(file) {
                Ok(l) => l,
                Err(e) => {
                    error!("Failed to load library: {e}");
                    Library::default()
                }
            }
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
        // End panicked processes
        self.any_process();

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

    /// Checks if there are any running operations on another thread.
    pub fn any_process(&mut self) -> bool {
        let mut res = false;

        if let Some(p) = &self.load_process {
            if p.handle.is_finished() {
                if let Err(e) = self.finish_get_new_songs() {
                    error!("Failed to get new songs: {e}");
                }
            } else {
                res = true;
            }
        }

        if let Some(p) = &self.save_process {
            if p.is_finished() {
                if let Err(e) = self.finish_save_songs() {
                    error!("Failed to save songs: {e}");
                }
            } else {
                res = true;
            }
        }

        if let Some(p) = &self.image_load_process {
            if p.is_finished() {
                if let Err(e) = self.finish_save_songs() {
                    error!("Failed to save songs: {e}");
                }
            }
        }

        if let Some(p) = &self.image_shrink_process {
            if p.is_finished() {
                if let Err(e) = self.finish_save_songs() {
                    error!("Failed to save songs: {e}");
                }
            }
        }

        res
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

impl IndexMut<SongId> for Library {
    fn index_mut(&mut self, index: SongId) -> &mut Song {
        if index.0 >= self.songs().len() {
            &mut self.ghost
        } else {
            if self.songs()[index.0].is_deleted() {
                &mut self.ghost
            } else {
                &mut self.songs_mut()[index.0]
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
                match self
                    .library
                    .start_to_default_json(&self.config, self.sender.clone())
                {
                    Err(Error::InvalidOperation(_)) => {}
                    Err(e) => error!("Failed to start library save: {e}"),
                    _ => {}
                }
            }
            Message::SaveEnded => {
                if let Err(e) = self.library.finish_save_songs() {
                    error!("Failed to finsih saving songs: {e}")
                }
            }
            Message::ImageLoadEnded => {
                if let Err(e) = self.library.finish_load_images() {
                    error!("Failed to load images: {e}");
                } else {
                    if let Err(e) = self.library.start_make_thumbnails(
                        self.sender.clone(),
                        &self.config,
                    ) {
                        error!("Failed to start make thumbnails: {e}");
                    }
                }
            }
            Message::ImageShrinkEnded => {
                if let Err(e) = self.library.finish_make_thumbnails() {
                    error!("Failed to make thumbnails: {e}");
                }
            }
        }
        ComMsg::none()
    }

    pub fn library_lib_update(&mut self) -> LibraryUpdate {
        let up = mem::replace(&mut self.library.lib_update, LibraryUpdate::None);

        if up >= LibraryUpdate::NewData {
            self.library.new_images = true;
        }

        if self.library.image_load_process.is_none() && self.library.new_images {
            self.library.new_images = false;
            if let Err(e) = self.library.start_load_images(self.sender.clone(), &self.config) {
                error!("Failed to start load for new images: {e}");
            }
        }

        up
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
                self.update(LibraryUpdate::NewData);
            }
            Ok(())
        } else {
            Err(Error::InvalidOperation("No load was running"))
        }
    }

    /// Finishes the loading of songs started by `start_get_new_songs`
    fn finish_save_songs(&mut self) -> Result<()> {
        if let Some(p) = self.save_process.take() {
            match p.join().map_err(|_| Error::ThreadPanicked).and_then(|e| e) {
                Err(e) => {
                    self.change.set(true);
                    Err(e)
                }
                Ok(_) => Ok(()),
            }
        } else {
            Err(Error::InvalidOperation("No load was running"))
        }
    }

    fn finish_load_images(&mut self) -> Result<()> {
        if let Some(p) = self.image_load_process.take() {
            let lib = p.join().map_err(|_| Error::ThreadPanicked)?;
            self.images = lib.images;
        }

        Ok(())
    }

    fn finish_make_thumbnails(&mut self) -> Result<()> {
        if let Some(p) = self.image_shrink_process.take() {
            self.images = p.join().map_err(|_| Error::ThreadPanicked)?;
        }

        Ok(())
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

    fn load_images(&mut self, conf: &Config) {
        let mut images = self.images.clone();
        let full_path = conf.image_cache().as_ref().map(|p| p.join("full"));
        let small_path = conf.image_cache().as_ref().map(|p| p.join("small"));

        for s in self.songs() {
            if !images.contains_key(&(s.artist().into(), s.album().into())) {
                let t = match Tag::new().read_from_path(s.path()) {
                    Ok(t) => t,
                    Err(e) => {
                        warn!("Failed to read tag: {e}");
                        continue;
                    }
                };

                let full = if let Some(mut f) =
                    full_path.as_ref().and_then(|p| {
                        File::open(p.join(valid_filename(
                            format!("/{} - {}", s.artist(), s.album()).chars(),
                        )))
                        .ok()
                    }) {
                    let mut buf = Vec::new();
                    if let Err(e) = f.read_to_end(&mut buf) {
                        warn!("Failed to read image: {e}");
                        continue;
                    }
                    Handle::from_memory(buf)
                } else if let Some(c) = t.album_cover() {
                    Handle::from_memory(
                        c.data.iter().map(|c| *c).collect_vec(),
                    )
                } else {
                    continue;
                };

                let small = if let Some(mut f) =
                    small_path.as_ref().and_then(|p| {
                        File::open(p.join(valid_filename(
                            format!("/{} - {}", s.artist(), s.album()).chars(),
                        )))
                        .ok()
                    }) {
                    let mut buf = Vec::new();
                    if let Err(e) = f.read_to_end(&mut buf) {
                        warn!("Failed to read thumbnail image: {e}");
                    }
                    Some(Handle::from_memory(buf))
                } else {
                    None
                };

                let key = (
                    s.artist().to_owned().into(),
                    s.album().to_owned().into(),
                );

                images.insert(key, CoverImage::new(full, small));
            }
        }

        self.images = images;
    }

    fn make_thumbnails(images: &mut ImageMap, conf: &Config) -> Result<()> {
        let small_path = conf.image_cache().as_ref().map(|p| p.join("small"));
        if let Some(path) = &small_path {
            create_dir_all(path)?;
        }

        for ((artist, album), img) in images.iter_mut() {
            if img.as_small().is_some() {
                continue;
            }

            if let Some(img) = img.make_thumbnail() {
                if let (Some(path), Data::Bytes(b)) = (&small_path, img.data())
                {
                    let data = b.as_ref();
                    let mut f = File::create(path.join(valid_filename(
                        format!("/{} - {}", artist, album).chars(),
                    )))?;
                    f.write_all(data)?;
                }
            }
        }

        Ok(())
    }
}

impl Clone for Library {
    fn clone(&self) -> Self {
        Self {
            songs: self.songs.clone(),
            load_process: None,
            save_process: None,
            image_load_process: None,
            image_shrink_process: None,
            lib_update: LibraryUpdate::None,
            new_images: self.new_images,
            ghost: self.ghost.clone(),
            change: self.change.clone(),
            images: self.images.clone(),
        }
    }
}

fn default_ghost() -> Song {
    Song::invalid()
}
