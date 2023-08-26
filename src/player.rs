use std::{
    fs::{create_dir_all, File},
    ops::{Index, IndexMut},
    path::Path,
    slice::{Iter, SliceIndex},
    sync::Arc
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

/// Wrapps the sink
struct SinkWrapper {
    sink: Sink,
}

impl SinkWrapper {
    /// Try to create new [`SinkWrapper`]
    ///
    /// # Errors
    /// - Failed to create the [`Sink`]
    pub fn try_new() -> Result<Self> {
        Ok(Self {
            sink: Sink::default_out()?,
        })
    }

    /// Plays the given song
    ///
    /// # Errors
    /// - only with synchronization, shouldn't happen
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

    /// Sets the play state
    ///
    /// # Errors
    /// - only with synchronization, shouldn't happen
    pub fn play_pause(&mut self, play: bool) -> Result<()> {
        self.sink.play(play)
    }

    /// Sets callback for when a song ends
    ///
    /// # Errors
    /// - only with synchronization, shouldn't happen
    pub fn on_song_end<F: FnMut() + Send + 'static>(
        &mut self,
        mut f: F,
    ) -> Result<()>
    where
        &'static F: std::marker::Send,
    {
        self.sink.on_callback(Some(move |cb| match cb {
            CallbackInfo::SourceEnded => {
                f();
            }
            _ => {}
        }))
    }

    /// Sets the playback volume
    ///
    /// # Errors
    /// - only with synchronization, shouldn't happen
    pub fn set_volume(&mut self, volume: f32) -> Result<()> {
        self.sink.volume(volume * volume)
    }
}

/// Messages sent by the player
#[derive(Clone, Copy, Debug)]
pub enum PlayerMessage {
    SongEnd,
}

/// State of the player playback
#[derive(Clone, Copy, Default)]
pub enum Playback {
    /// No song is playing
    #[default]
    Stopped,
    /// Song is playing
    Playing,
    /// Song is paused
    Paused,
}

/// A playlist, lazily cloned
pub enum Playlist {
    /// There is static immutable reference to the playlist
    Static(Arc<[SongId]>),
    /// There is owned vector with the playlist
    Dynamic(Vec<SongId>),
}

/// Contains either sink or a sender for callbacks
enum MaybeSink {
    /// It is initialized sink
    Sink(SinkWrapper),
    /// It is only the sender
    Sender(Arc<UnboundedSender<UampMessage>>),
}

/// Player for uamp, manages playlist
pub struct Player {
    /// The inner player if initialized
    inner: MaybeSink,
    /// The playback state
    state: Playback,
    /// The current playlist
    playlist: Playlist,
    /// The current song or [`None`]
    current: Option<usize>,
    /// The volume of the playback, doesn't affect mute
    volume: f32,
    /// True when the sound is muted, doesn't affect volume
    mute: bool,
}

/// Used for deserializing the data of the [`Player`]
#[derive(Deserialize)]
struct PlayerDataLoad {
    /// True when the sound is muted, doesn't affect volume
    #[serde(default)]
    mute: bool,
    /// The volume of the playback, doesn't affect mute
    #[serde(default = "default_volume")]
    volume: f32,
    /// The current song or [`None`]
    #[serde(default)]
    current: Option<usize>,
    /// The current playlist
    #[serde(default)]
    playlist: Playlist,
}

/// returns the default volume
fn default_volume() -> f32 {
    1.
}

/// Used for serializing the data of the [`Player`]
#[derive(Serialize)]
struct PlayerDataSave<'a> {
    /// True when the sound is muted, doesn't affect volume
    mute: bool,
    /// The volume of the playback, doesn't affect mute
    volume: f32,
    /// The current song or [`None`]
    current: Option<usize>,
    /// The current playlist
    playlist: &'a Playlist,
}

impl Player {
    /// Handles player event messages
    pub fn event(&mut self, lib: &Library, msg: PlayerMessage) -> Command {
        match msg {
            PlayerMessage::SongEnd => {
                self.play_next(lib);
            }
        }
        Command::none()
    }

    /// Tries the create inner player if it is not created
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
                        });
                        if self.mute {
                            _ = p.set_volume(0.);
                        } else {
                            _ = p.set_volume(self.volume);
                        }
                    }
                    _ => {} // this will never happen
                }
            }
        }
    }

    /// Sets the playback state to play/pause
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

    /// Tries to load a song into the library. Stops on failure.
    pub fn try_load(
        &mut self,
        lib: &Library,
        index: Option<usize>,
        play: bool,
    ) {
        match index {
            Some(i) if i < self.playlist.len() => self.load(lib, i, play),
            _ => {
                self.stop()
            }
        }
    }

    /// Loads a song into the player.
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

    /// Toggles the states play/pause, also plays if the state is Stopped and
    /// current is [`Some`]
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

    /// Gets the current volume.
    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// Sets the playback volume. It is not set on error.
    pub fn set_volume(&mut self, volume: f32) {
        self.try_get_player();
        if let MaybeSink::Sink(s) = &mut self.inner {
            if matches!(s.set_volume(volume * volume), Ok(_)) {
                self.volume = volume;
            }
        }
    }

    /// Gets the mute state
    pub fn mute(&self) -> bool {
        self.mute
    }

    /// Sets the mute state. It is not set on error.
    pub fn set_mute(&mut self, mute: bool) {
        self.try_get_player();
        if let MaybeSink::Sink(s) = &mut self.inner {
            let r = if mute {
                s.set_volume(0.)
            } else {
                s.set_volume(self.volume)
            };
            if r.is_ok() {
                self.mute = mute;
            }
        }
    }

    /// Toggle mute, not toggled on error.
    pub fn toggle_mute(&mut self) {
        self.set_mute(!self.mute)
    }

    /// Loads the given playlist at the given index
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

    /// Returns true if the state is [`Playback::Playing`]
    pub fn is_playing(&self) -> bool {
        matches!(self.state, Playback::Playing)
    }

    /// gets the now playing song if available
    pub fn now_playing(&self) -> Option<SongId> {
        self.current.map(|i| self.playlist[i])
    }

    /// Plays the next song in the playlist
    pub fn play_next(&mut self, lib: &Library) {
        self.jump_to(lib, self.current.map(|i| i + 1))
    }

    /// Plays the previous song in the playlist
    pub fn play_prev(&mut self, lib: &Library) {
        self.jump_to(lib, self.current.and_then(|i| i.checked_sub(1)))
    }

    /// Jumps to the given index in the playlist if set.
    pub fn jump_to(&mut self, lib: &Library, index: Option<usize>) {
        self.try_load(lib, index, self.is_playing())
    }

    /// Gets the playlist
    pub fn playlist(&self) -> &Playlist {
        &self.playlist
    }

    /// Jumps to the given index in the playlist.
    pub fn play_at(&mut self, lib: &Library, index: usize, play: bool) {
        if index >= self.playlist.len() {
            self.stop();
            return;
        }

        self.load(lib, index, play);
    }

    /// Changes the state to [`Playback::Stopped`]
    pub fn stop(&mut self) {
        self.try_get_player();
        if let MaybeSink::Sink(p) = &self.inner {
            if p.sink.play(false).is_ok() {
                self.current = None;
                self.state = Playback::Stopped;
            }
        }
    }

    /// Shuffles the playlist, the current song is still the same song
    pub fn shuffle(&mut self) {
        let id = self.now_playing();
        self.playlist[..].shuffle(&mut thread_rng());
        // find the currently playing in the shuffled playlist
        if let Some(id) = id {
            self.current = self.playlist.iter().position(|i| *i == id);
        }
    }

    /// Loads the playback state from json based on the config, returns default
    /// [`Player`] on fail
    pub fn from_config(
        sender: Arc<UnboundedSender<UampMessage>>,
        conf: &Config,
    ) -> Self {
        Self::from_json(sender, &conf.player_path)
    }

    /// Saves the playback state to the given json file
    ///
    /// # Errors
    /// - cannot create parrent directory
    /// - Failed to serialize
    pub fn to_json(&self, path: impl AsRef<Path>) -> Result<()> {
        if let Some(par) = path.as_ref().parent() {
            create_dir_all(par)?;
        }

        serde_json::to_writer(
            File::create(path)?,
            &PlayerDataSave {
                playlist: &self.playlist,
                current: self.current,
                volume: self.volume,
                mute: self.mute,
            },
        )?;
        Ok(())
    }

    /// Loads the playback state from the given json file, returns default
    /// [`Player`] on fail
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
            volume: data.volume,
            mute: data.mute,
        }
    }
}

impl Playlist {
    /// Gets the number of items in the playlist
    pub fn len(&self) -> usize {
        match self {
            Self::Static(s) => s.len(),
            Self::Dynamic(d) => d.len(),
        }
    }

    /// Clones the playlist and creates owned vector
    #[inline]
    fn make_dynamic(&mut self) {
        if let Self::Static(s) = self {
            *self = Self::Dynamic(s.as_ref().into())
        }
    }

    /// Returns iterator over the items
    pub fn iter<'a>(&'a self) -> Iter<'_, SongId> {
        self[..].iter()
    }

    /// Gets/Creates arc from the playlist
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

impl Default for PlayerDataLoad {
    fn default() -> Self {
        Self {
            mute: false,
            volume: default_volume(),
            current: None,
            playlist: [].as_slice().into()
        }
    }
}
