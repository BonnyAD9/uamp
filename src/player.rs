use std::{
    fs::{create_dir_all, File},
    ops::{Index, IndexMut},
    path::Path,
    slice::{Iter, SliceIndex},
    sync::Arc,
};

use eyre::Result;
use log::info;
use rand::{seq::SliceRandom, thread_rng};
use raplay::{
    sink::{CallbackInfo, Sink},
    source::symph::Symph,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    config::Config,
    library::{Library, SongId},
    uamp_app::UampMessage,
    wid::Command,
};

struct SinkWrapper {
    sink: Sink,
}

impl SinkWrapper {
    pub fn try_new() -> Result<Self> {
        Ok(Self {
            sink: Sink::default_out()?,
        })
    }

    pub fn play(
        &mut self,
        lib: &Library,
        id: SongId,
        play: bool,
    ) -> Result<()> {
        let file = File::open(lib[id].path())?;
        let src = Symph::try_new(file)?;
        self.sink.load(src, play)
    }

    pub fn play_pause(&mut self, play: bool) -> Result<()> {
        self.sink.play(play)
    }

    pub fn on_song_end<F: FnMut() + Send + 'static>(
        &mut self,
        mut f: F,
    ) -> Result<()>
    where
        &'static F: std::marker::Send,
    {
        self.sink.on_callback(Some(move |cb| match cb {
            CallbackInfo::SourceEnded => f(),
            _ => {}
        }))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PlayerMessage {
    SongEnd,
}

#[derive(Clone, Copy, Default)]
pub enum Playback {
    #[default]
    Stopped,
    Playing,
    Paused,
}

pub enum Playlist {
    Static(Arc<[SongId]>),
    Dynamic(Vec<SongId>),
}

enum MaybeSink {
    Sink(SinkWrapper),
    Sender(Arc<UnboundedSender<UampMessage>>),
}

pub struct Player {
    inner: MaybeSink,
    state: Playback,
    playlist: Playlist,
    current: Option<usize>,
}

#[derive(Default, Deserialize)]
struct PlayerDataLoad {
    current: Option<usize>,
    playlist: Playlist,
}

#[derive(Serialize)]
struct PlayerDataSave<'a> {
    current: Option<usize>,
    playlist: &'a Playlist,
}

impl Player {
    pub fn new(sender: Arc<UnboundedSender<UampMessage>>) -> Self {
        Self {
            inner: MaybeSink::Sender(sender),
            state: Playback::Stopped,
            playlist: [][..].into(),
            current: None,
        }
    }

    pub fn event(&mut self, lib: &Library, msg: PlayerMessage) -> Command {
        match msg {
            PlayerMessage::SongEnd => self.play_next(lib),
        }
        Command::none()
    }

    fn try_get_player(&mut self) {
        if matches!(self.inner, MaybeSink::Sender(_)) {
            if let Ok(inner) = SinkWrapper::try_new() {
                let sender =
                    std::mem::replace(&mut self.inner, MaybeSink::Sink(inner));
                match (&mut self.inner, sender) {
                    (MaybeSink::Sink(p), MaybeSink::Sender(s)) => {
                        _ = p.on_song_end(move || {
                            _ = s.send(UampMessage::Player(
                                PlayerMessage::SongEnd,
                            ));
                        })
                    }
                    _ => {} // this will never happen
                }
            }
        }
    }

    pub fn play(&mut self, play: bool) {
        self.try_get_player();
        if let MaybeSink::Sink(p) = &mut self.inner {
            self.state = if play {
                Playback::Playing
            } else {
                Playback::Paused
            };

            _ = p.play_pause(play);
        }
    }

    pub fn try_load(
        &mut self,
        lib: &Library,
        index: Option<usize>,
        play: bool,
    ) {
        match index {
            Some(i) if i < self.playlist.len() => self.load(lib, i, play),
            _ => {
                self.current = None;
                self.state = Playback::Stopped;
            }
        }
    }

    /// doesn't check that the index is valid
    fn load(&mut self, lib: &Library, index: usize, play: bool) {
        self.current = Some(index);

        self.try_get_player();
        let inner = match &mut self.inner {
            MaybeSink::Sink(i) => i,
            _ => {
                self.state = Playback::Stopped;
                return;
            }
        };

        self.state = if play {
            Playback::Playing
        } else {
            Playback::Paused
        };

        _ = inner.play(lib, self.playlist[index], play);
    }

    pub fn play_pause(&mut self, lib: &Library) {
        match self.state {
            Playback::Stopped => {
                if let Some(id) = self.current {
                    self.load(lib, id, true);
                }
            }
            Playback::Playing => {
                self.play(false);
            }
            Playback::Paused => {
                self.play(true);
            }
        }
    }

    pub fn play_playlist(
        &mut self,
        lib: &Library,
        songs: impl Into<Playlist>,
        index: Option<usize>,
        play: bool,
    ) {
        self.playlist = songs.into();
        self.try_load(lib, index, play);
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.state, Playback::Playing)
    }

    pub fn now_playing(&self) -> Option<SongId> {
        self.current.map(|i| self.playlist[i])
    }

    pub fn play_next(&mut self, lib: &Library) {
        match self.state {
            Playback::Stopped => {}
            Playback::Playing => {
                self.try_load(lib, self.current.map(|i| i + 1), true);
            }
            Playback::Paused => {
                self.try_load(lib, self.current.map(|i| i + 1), false)
            }
        }
    }

    pub fn playlist(&self) -> &Playlist {
        &self.playlist
    }

    pub fn play_at(&mut self, lib: &Library, index: usize, play: bool) {
        if index >= self.playlist.len() {
            self.current = None;
            self.state = Playback::Stopped;
            return;
        }

        self.load(lib, index, play);
    }

    pub fn shuffle(&mut self) {
        let id = self.now_playing();
        self.playlist[..].shuffle(&mut thread_rng());
        // find the currently playing in the shuffled playlist
        if let Some(id) = id {
            self.current = self.playlist.iter().position(|i| *i == id);
        }
    }

    pub fn from_config(
        sender: Arc<UnboundedSender<UampMessage>>,
        conf: &Config,
    ) -> Self {
        Self::from_json(sender, &conf.player_path)
    }

    pub fn to_json(&self, path: impl AsRef<Path>) -> Result<()> {
        if let Some(par) = path.as_ref().parent() {
            create_dir_all(par)?;
        }

        serde_json::to_writer(
            File::create(path)?,
            &PlayerDataSave {
                playlist: &self.playlist,
                current: self.current,
            },
        )?;
        Ok(())
    }

    pub fn from_json(
        sender: Arc<UnboundedSender<UampMessage>>,
        path: impl AsRef<Path>,
    ) -> Self {
        let data = if let Ok(file) = File::open(path.as_ref()) {
            serde_json::from_reader(file).unwrap_or_default()
        } else {
            info!("player file {:?} doesn't exist", path.as_ref());
            PlayerDataLoad::default()
        };

        Self {
            inner: MaybeSink::Sender(sender),
            state: Playback::Stopped,
            playlist: data.playlist,
            current: data.current,
        }
    }
}

impl Playlist {
    pub fn len(&self) -> usize {
        match self {
            Self::Static(s) => s.len(),
            Self::Dynamic(d) => d.len(),
        }
    }

    #[inline]
    fn make_dynamic(&mut self) {
        if let Self::Static(s) = self {
            *self = Self::Dynamic(s.as_ref().into())
        }
    }

    /// This will panic when it is static playlist
    fn mut_dyn(&mut self) -> &mut Vec<SongId> {
        match self {
            Playlist::Static(_) => panic!(),
            Playlist::Dynamic(d) => d,
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'_, SongId> {
        self[..].iter()
    }

    pub fn as_arc(&self) -> Arc<[SongId]> {
        match self {
            Playlist::Static(a) => a.clone(),
            // copy to arc when this is vector
            Playlist::Dynamic(v) => v[..].into(),
        }
    }
}

impl Default for Playlist {
    fn default() -> Self {
        [][..].into()
    }
}

impl<T: SliceIndex<[SongId]>> Index<T> for Playlist {
    type Output = T::Output;

    fn index(&self, index: T) -> &Self::Output {
        match self {
            Self::Static(s) => s.index(index),
            Self::Dynamic(d) => d.index(index),
        }
    }
}

impl<T: SliceIndex<[SongId]>> IndexMut<T> for Playlist {
    #[inline]
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        match self {
            Self::Static(_) => {
                self.make_dynamic();
                self.index_mut(index)
            }
            Self::Dynamic(d) => d.index_mut(index),
        }
    }
}

impl From<&[SongId]> for Playlist {
    fn from(value: &[SongId]) -> Self {
        Self::Dynamic(value.into())
    }
}

impl From<Vec<SongId>> for Playlist {
    fn from(value: Vec<SongId>) -> Self {
        Self::Dynamic(value)
    }
}

impl From<Arc<[SongId]>> for Playlist {
    fn from(value: Arc<[SongId]>) -> Self {
        Self::Static(value)
    }
}

impl Serialize for Playlist {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Static(s) => s.as_ref().serialize(serializer),
            Self::Dynamic(v) => v.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Playlist {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::Dynamic(Vec::deserialize(deserializer)?))
    }

    fn deserialize_in_place<D>(
        deserializer: D,
        place: &mut Self,
    ) -> std::result::Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match place {
            Playlist::Static(_) => {
                *place = Self::Dynamic(Vec::deserialize(deserializer)?);
                Ok(())
            }
            Playlist::Dynamic(v) => Vec::deserialize_in_place(deserializer, v),
        }
    }
}
