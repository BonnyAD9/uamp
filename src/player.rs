use std::{
    cell::Cell,
    fs::{create_dir_all, File},
    ops::{Index, IndexMut},
    path::Path,
    slice::{Iter, SliceIndex},
    sync::Arc,
    time::Duration,
};

use log::{error, info, warn};
use rand::{seq::SliceRandom, thread_rng};
use raplay::{
    sink::{CallbackInfo, Sink},
    source::symph::Symph,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    config::Config,
    err::Result,
    library::{Library, SongId},
    uamp_app::UampMessage,
    wid::Command, gen_struct,
};

gen_struct! {
    pub Player {
        playlist: Playlist { pub, pri },
        ;
        current: Option<usize> { pri, pri },
        volume: f32 { pub, pri },
        mute: bool { pub, pri },
        ;
        state: Playback,
        inner: SinkWrapper,
    }
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

    /// Sets the playback state to play/pause
    pub fn play(&mut self, play: bool) {
        if !self.state.is_stopped() {
            match self.inner.play(play) {
                Ok(_) => {
                    self.state = Playback::play(play);
                }
                Err(e) => error!("Failed to play/pause: {}", e),
            }
            return;
        }

        match self
            .inner
            .seek_to(Duration::ZERO)
            .and_then(|_| self.inner.play(play))
        {
            Ok(_) => self.state = Playback::play(play),
            Err(e) => warn!("Failed to resume from stop: {}", e),
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
            Some(i) if i < self.playlist().len() => self.load(lib, i, play),
            _ => self.stop(),
        }
    }

    /// Loads a song into the player.
    /// doesn't check that the index is valid
    fn load(&mut self, lib: &Library, index: usize, play: bool) {
        self.current_set(Some(index));

        match self.inner.load(lib, self.playlist()[index], play) {
            Ok(_) => self.state = Playback::play(play),
            Err(e) => error!("Failed to load song: {e}"),
        }
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

    /// Sets the playback volume. It is not set on error.
    pub fn set_volume(&mut self, volume: f32) {
        match self.inner.set_volume(volume) {
            Ok(_) => self.volume_set(volume),
            Err(e) => error!("Failed to set volume: {}", e),
        }
    }

    /// Sets the mute state. It is not set on error.
    pub fn set_mute(&mut self, mute: bool) {
        let vol = if mute { 0. } else { self.volume() };

        match self.inner.set_volume(vol) {
            Ok(_) => self.mute_set(mute),
            Err(e) => error!("Failed to mute: {}", e),
        }
    }

    /// Toggle mute, not toggled on error.
    pub fn toggle_mute(&mut self) {
        self.set_mute(!self.mute())
    }

    /// Loads the given playlist at the given index
    pub fn play_playlist(
        &mut self,
        lib: &Library,
        songs: impl Into<Playlist>,
        index: Option<usize>,
        play: bool,
    ) {
        *self.playlist_mut() = songs.into();
        self.try_load(lib, index, play);
    }

    /// Returns true if the state is [`Playback::Playing`]
    pub fn is_playing(&self) -> bool {
        matches!(self.state, Playback::Playing)
    }

    /// gets the now playing song if available
    pub fn now_playing(&self) -> Option<SongId> {
        self.current.map(|i| self.playlist()[i])
    }

    /// Plays the next song in the playlist
    pub fn play_next(&mut self, lib: &Library) {
        self.jump_to(lib, self.current().map(|i| i + 1))
    }

    /// Plays the previous song in the playlist
    pub fn play_prev(&mut self, lib: &Library) {
        self.jump_to(lib, self.current().and_then(|i| i.checked_sub(1)))
    }

    /// Jumps to the given index in the playlist if set.
    pub fn jump_to(&mut self, lib: &Library, index: Option<usize>) {
        self.try_load(lib, index, self.is_playing())
    }

    /// Jumps to the given index in the playlist.
    pub fn play_at(&mut self, lib: &Library, index: usize, play: bool) {
        if index >= self.playlist().len() {
            self.stop();
            return;
        }

        self.load(lib, index, play);
    }

    /// Changes the state to [`Playback::Stopped`]
    pub fn stop(&mut self) {
        match self.inner.play(false) {
            Ok(_) => self.state = Playback::Stopped,
            Err(e) => error!("Failed to stop playback: {}", e),
        }
    }

    /// Shuffles the playlist, the current song is still the same song
    pub fn shuffle(&mut self) {
        let id = self.now_playing();
        self.playlist_mut()[..].shuffle(&mut thread_rng());
        // find the currently playing in the shuffled playlist
        if let Some(id) = id {
            self.current = self.playlist().iter().position(|i| *i == id);
        }
    }

    /// Loads the playback state from json based on the config, returns default
    /// [`Player`] on fail
    pub fn from_config(
        sender: Arc<UnboundedSender<UampMessage>>,
        conf: &Config,
    ) -> Self {
        Self::from_json(sender, conf.player_path())
    }

    /// Saves the playback state to the given json file
    ///
    /// # Errors
    /// - cannot create parrent directory
    /// - Failed to serialize
    fn to_json(&self, path: impl AsRef<Path>) -> Result<()> {
        if let Some(par) = path.as_ref().parent() {
            create_dir_all(par)?;
        }

        serde_json::to_writer(
            File::create(path)?,
            &PlayerDataSave {
                playlist: self.playlist(),
                current: self.current(),
                volume: self.volume(),
                mute: self.mute(),
            },
        )?;
        Ok(())
    }

    /// Saves the playback state to the default json directory. It doesn't
    /// save the data if it didn't change since the last save.
    ///
    /// # Errors
    /// - cannot create parrent directory
    /// - Failed to serialize
    pub fn to_default_json(&self, conf: &Config) -> Result<()> {
        if !self.change.get() {
            return Ok(());
        }
        self.to_json(conf.player_path())?;
        self.change.set(false);
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

        let mut sink = SinkWrapper::new();
        if let Err(e) = sink.on_song_end(move || {
            if let Err(e) =
                sender.send(UampMessage::Player(PlayerMessage::SongEnd))
            {
                error!("Failed to inform song end: {e}");
            }
        }) {
            error!("Failed to set the song end callback: {e}");
        }
        let volume = if let Err(e) = sink.set_volume(data.volume) {
            error!("Failed to set the initial volume: {e}");
            1. // 1 is the default volume of the player
        } else {
            data.volume
        };

        Self {
            inner: sink,
            state: Playback::Stopped,
            playlist: data.playlist,
            current: data.current,
            volume,
            mute: data.mute,
            change: Cell::new(false),
        }
    }
}

/// Wrapps the sink
struct SinkWrapper(Sink);

impl SinkWrapper {
    /// Create new [`SinkWrapper`]
    pub fn new() -> Self {
        Self(Sink::default())
    }

    /// Plays the given song
    ///
    /// # Errors
    /// - only with synchronization, shouldn't happen
    pub fn load(
        &mut self,
        lib: &Library,
        id: SongId,
        play: bool,
    ) -> Result<()> {
        let file = File::open(lib[id].path())?;
        let src = Symph::try_new(file, &Default::default())?;
        self.0.load(src, play)?;
        Ok(())
    }

    /// Sets the play state
    ///
    /// # Errors
    /// - only with synchronization, shouldn't happen
    pub fn play(&mut self, play: bool) -> Result<()> {
        self.0.play(play)?;
        Ok(())
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
        self.0.on_callback(Some(move |cb| match cb {
            CallbackInfo::SourceEnded => {
                f();
            }
            _ => {}
        }))?;
        Ok(())
    }

    /// Sets the playback volume
    ///
    /// # Errors
    /// - only with synchronization, shouldn't happen
    pub fn set_volume(&mut self, volume: f32) -> Result<()> {
        self.0.volume(volume * volume)?;
        Ok(())
    }

    pub fn seek_to(&mut self, pos: Duration) -> Result<()> {
        self.0.seek_to(pos)?;
        Ok(())
    }
}

/// A playlist, lazily cloned
pub enum Playlist {
    /// There is static immutable reference to the playlist
    Static(Arc<[SongId]>),
    /// There is owned vector with the playlist
    Dynamic(Vec<SongId>),
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

impl Playback {
    pub fn play(play: bool) -> Self {
        if play {
            Self::Playing
        } else {
            Self::Paused
        }
    }

    pub fn is_stopped(&self) -> bool {
        matches!(self, Playback::Stopped)
    }
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

impl Default for PlayerDataLoad {
    fn default() -> Self {
        Self {
            mute: false,
            volume: default_volume(),
            current: None,
            playlist: [].as_slice().into(),
        }
    }
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

/// returns the default volume
fn default_volume() -> f32 {
    1.
}
