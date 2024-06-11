pub mod add_policy;
mod msg;
mod playback;
mod sink_wrapper;

pub use self::msg::Message as PlayerMessage;
use self::{
    add_policy::AddPolicy, msg::Message, playback::Playback,
    sink_wrapper::SinkWrapper,
};

use std::{
    cell::Cell,
    fs::{create_dir_all, File},
    mem,
    path::Path,
    time::Duration,
};

use futures::channel::mpsc::UnboundedSender;
use log::{error, info, warn};
use rand::{seq::SliceRandom, thread_rng};
use raplay::{CallbackInfo, Timestamp};
use serde::{Deserialize, Serialize};

use crate::{
    app::UampApp,
    config::Config,
    core::{alc_vec::AlcVec, msg::Msg, Result},
    gen_struct,
    library::{Library, LibraryUpdate, SongId},
};

gen_struct! {
    pub Player {
        // Reference
        playlist: AlcVec<SongId> { pub, pub },
        ; // value
        current: Option<usize> { pub, pri },
        volume: f32 { pub, pri },
        mute: bool { pub, pri },
        ; // other
        intercept: Option<PlaylistInfo>,
        pub shuffle_current: bool,
        state: Playback,
        inner: SinkWrapper,
    }
}

#[derive(Serialize, Deserialize)]
pub struct PlaylistInfo {
    playlist: AlcVec<SongId>,
    current: Option<usize>,
    position: Option<Duration>,
}

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

impl Player {
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
        lib: &mut Library,
        index: Option<usize>,
        play: bool,
    ) -> bool {
        match index {
            Some(i) if i < self.playlist().len() => {
                self.load(lib, i, play);
                true
            }
            _ => {
                self.stop();
                false
            }
        }
    }

    /// Toggles the states play/pause, also plays if the state is Stopped and
    /// current is [`Some`]
    pub fn play_pause(&mut self, lib: &mut Library, play: bool) {
        match self.state {
            Playback::Stopped => {
                if let Some(id) = self.current {
                    self.load(lib, id, play);
                }
            }
            _ => self.play(play),
        }
    }

    /// Sets the playback volume. It is not set on error.
    pub fn set_volume(&mut self, volume: f32) {
        match self.inner.set_volume(volume) {
            Ok(_) => _ = self.volume_set(volume),
            Err(e) => error!("Failed to set volume: {}", e),
        }
    }

    /// Sets the mute state. It is not set on error.
    pub fn set_mute(&mut self, mute: bool) {
        let vol = if mute { 0. } else { self.volume() };

        match self.inner.set_volume(vol) {
            Ok(_) => _ = self.mute_set(mute),
            Err(e) => error!("Failed to mute: {}", e),
        }
    }

    /// Loads the given playlist at the given index
    pub fn play_playlist(
        &mut self,
        lib: &mut Library,
        songs: impl Into<AlcVec<SongId>>,
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

    /// Plays the `n`th next song in the playlist
    pub fn play_next(&mut self, lib: &mut Library, n: usize) {
        let s = self.state.is_stopped();
        self.jump_to(lib, self.current().map(|i| i + n));
        if !s & self.state.is_stopped() {
            self.current = if self.playlist.len() == 0 {
                None
            } else {
                Some(0)
            }
        }
    }

    /// Plays the `n`th previous song in the playlist
    pub fn play_prev(&mut self, lib: &mut Library, n: usize) {
        self.jump_to(lib, self.current().and_then(|i| i.checked_sub(n)))
    }

    /// Jumps to the given index in the playlist if set.
    pub fn jump_to(&mut self, lib: &mut Library, index: Option<usize>) {
        self.try_load(lib, index, self.is_playing());
    }

    /// Jumps to the given index in the playlist.
    pub fn play_at(&mut self, lib: &mut Library, index: usize, play: bool) {
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
            if !self.shuffle_current {
                if let Some(c) = self.current {
                    self.playlist_mut()[..].swap(c, 0);
                    self.current = Some(0);
                }
            }
        }
    }

    pub fn intercept(
        &mut self,
        lib: &mut Library,
        songs: impl Into<AlcVec<SongId>>,
        index: Option<usize>,
        play: bool,
    ) {
        if self.intercept.is_some() {
            self.play_playlist(lib, songs, index, play);
            return;
        }

        let Some(index) = index else {
            return;
        };
        let songs = songs.into();
        if index >= songs.len() {
            return;
        }

        self.intercept = Some(PlaylistInfo {
            playlist: mem::replace(self.playlist_mut(), songs),
            current: self.current,
            position: self.timestamp().map(|t| t.current),
        });
        self.load(lib, index, play);
    }

    pub fn pop_intercept(&mut self, lib: &mut Library) {
        if let Some(ic) = self.intercept.take() {
            *self.playlist_mut() = ic.playlist;
            self.current_set(ic.current);
            if ic.current.is_some() {
                let play = self.is_playing();
                self.play_pause(lib, play);
                if let Some(p) = ic.position {
                    self.seek_to(p);
                }
            }
        }
    }

    /// Loads the playback state from json based on the config, returns default
    /// [`Player`] on fail
    pub fn from_config(
        lib: &mut Library,
        sender: UnboundedSender<Msg>,
        conf: &Config,
    ) -> Self {
        if let Some(p) = conf.player_path() {
            Self::from_json(lib, sender, p)
        } else {
            Self::new(sender)
        }
    }

    /// Saves the playback state to the default json directory. It doesn't
    /// save the data if it didn't change since the last save.
    ///
    /// # Errors
    /// - cannot create parrent directory
    /// - Failed to serialize
    pub fn to_default_json(&self, closing: bool, conf: &Config) -> Result<()> {
        let save_pos = conf.save_playback_pos().save(closing);
        if !save_pos && !self.change.get() {
            return Ok(());
        }
        if let Some(p) = conf.player_path() {
            self.to_json(save_pos, p)?;
        }
        self.change.set(false);
        Ok(())
    }

    /// Loads the playback state from the given json file, returns default
    /// [`Player`] on fail
    pub fn from_json(
        lib: &mut Library,
        sender: UnboundedSender<Msg>,
        path: impl AsRef<Path>,
    ) -> Self {
        let data = if let Ok(file) = File::open(path.as_ref()) {
            match serde_json::from_reader(file) {
                Ok(p) => p,
                Err(e) => {
                    error!("Failed to load playback info: {e}");
                    PlayerDataLoad::default()
                }
            }
        } else {
            info!("player file {:?} doesn't exist", path.as_ref());
            PlayerDataLoad::default()
        };

        let mut res = Self {
            inner: SinkWrapper::new(),
            state: Playback::Stopped,
            playlist: data.playlist,
            intercept: data.intercept,
            current: data.current,
            volume: data.volume,
            mute: data.mute,
            shuffle_current: true,
            change: Cell::new(false),
        };

        res.init_inner(sender);
        if let Some(p) = data.position {
            res.play_pause(lib, false);
            res.seek_to(p);
        }
        res
    }

    /// Sets the fade duration for play/pause
    pub fn fade_play_pause(&mut self, t: Duration) {
        if let Err(e) = self.inner.fade_play_pause(t) {
            error!("Failed to set the fade duration: {e}");
        }
    }

    /// Configures the player
    pub fn load_config(&mut self, conf: &Config) {
        self.fade_play_pause(conf.fade_play_pause().0);
        self.inner.set_gapless(conf.gapless());
        self.shuffle_current = conf.shuffle_current();
    }

    /// Gets timestamp of the current playback, returns [`None`] if nothing
    /// is playing
    pub fn timestamp(&self) -> Option<Timestamp> {
        if self.state.is_stopped() {
            None
        } else {
            self.inner.get_timestamp().ok()
        }
    }

    /// Seeks to the given position
    pub fn seek_to(&mut self, t: Duration) -> Option<Timestamp> {
        match self.inner.seek_to(t) {
            Err(e) => {
                error!("Failed to seek: {e}");
                None
            }
            Ok(ts) => Some(ts),
        }
    }

    pub fn seek_by(
        &mut self,
        t: Duration,
        forward: bool,
    ) -> Option<Timestamp> {
        match self.inner.seek_by(t, forward) {
            Err(e) => {
                error!("Failed to seek by: {e}");
                None
            }
            Ok(ts) => Some(ts),
        }
    }

    pub fn remove_deleted(&mut self, lib: &Library) {
        let cur = self.now_playing();
        self.playlist_mut()
            .vec_mut()
            .retain(|s| !lib[s].is_deleted());
        if let Some(cur) = cur {
            self.current_set(self.playlist.iter().position(|i| i == &cur));
        }
    }

    pub fn hard_pause(&mut self) {
        if let Err(e) = self.inner.hard_pause() {
            warn!("Failed to hard pause: {e}");
        }
    }

    pub fn add_songs<I>(&mut self, songs: I, policy: AddPolicy)
    where
        I: IntoIterator<Item = SongId>,
    {
        let i = self.current().unwrap_or_default() + 1;
        match policy {
            AddPolicy::End => self.playlist_mut().extend(songs),
            AddPolicy::Next => self.playlist_mut().splice(i..i, songs),
            AddPolicy::MixIn => self.playlist_mut().mix_after(i, songs),
        }
    }

    pub fn get_ids(&mut self) -> (AlcVec<SongId>, Option<AlcVec<SongId>>) {
        (
            self.playlist.clone(),
            self.intercept.as_mut().map(|a| a.playlist.clone()),
        )
    }
}

impl UampApp {
    /// Handles player event messages
    pub fn player_event(&mut self, msg: Message) -> Option<Msg> {
        match msg {
            Message::SongEnd => {
                self.player.play_next(&mut self.library, 1);
            }
            Message::HardPauseAt(i) => self.hard_pause_at = Some(i),
        }
        None
    }

    pub fn player_lib_update(&mut self, up: LibraryUpdate) {
        if up >= LibraryUpdate::RemoveData {
            self.player.remove_deleted(&self.library);
        }
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl Player {
    /// Loads a song into the player.
    /// doesn't check that the index is valid
    fn load(&mut self, lib: &mut Library, index: usize, play: bool) {
        self.current_set(Some(index));

        match self.inner.load(lib, self.playlist()[index], play) {
            Ok(_) => self.state = Playback::play(play),
            Err(e) => error!("Failed to load song: {e}"),
        }

        // TODO: skip to next

        if !play {
            self.hard_pause();
        }
    }

    /// Saves the playback state to the given json file
    ///
    /// # Errors
    /// - cannot create parrent directory
    /// - Failed to serialize
    fn to_json(&self, save_pos: bool, path: impl AsRef<Path>) -> Result<()> {
        if let Some(par) = path.as_ref().parent() {
            create_dir_all(par)?;
        }

        let position = save_pos
            .then(|| self.timestamp())
            .flatten()
            .map(|t| t.current);

        serde_json::to_writer(
            File::create(path)?,
            &PlayerDataSave {
                playlist: self.playlist(),
                current: self.current(),
                volume: self.volume(),
                mute: self.mute(),
                position,
                intercept: self.intercept.as_ref(),
            },
        )?;
        Ok(())
    }

    /// Creates new player from the sender
    fn new(sender: UnboundedSender<Msg>) -> Self {
        let mut res = Self {
            playlist: AlcVec::new(),
            intercept: None,
            current: None,
            volume: default_volume(),
            mute: false,
            state: Playback::Stopped,
            inner: SinkWrapper::new(),
            change: Cell::new(true),
            shuffle_current: true,
        };

        res.init_inner(sender);
        res
    }

    fn song_end_handler(msg: CallbackInfo, sender: &UnboundedSender<Msg>) {
        let message = match msg {
            CallbackInfo::SourceEnded => Msg::Player(Message::SongEnd),
            CallbackInfo::PauseEnds(i) => Msg::Player(Message::HardPauseAt(i)),
            _ => todo!("Fix me at {}:{}::", file!(), line!()),
        };

        if let Err(e) = sender.unbounded_send(message) {
            error!("Failed to sink callback message: {e}");
        }
    }

    fn init_inner(&mut self, sender: UnboundedSender<Msg>) {
        if let Err(e) = self
            .inner
            .on_callback(move |msg| Self::song_end_handler(msg, &sender))
        {
            error!("Failed to set the song end callback: {e}");
        }

        (self.mute, self.volume) = if self.mute {
            if let Err(e) = self.inner.set_volume(0.) {
                error!("Failed to set the initial volume: {e}");
                (false, 1.) // 1 is the default volume of the player
            } else {
                (true, self.volume)
            }
        } else if let Err(e) = self.inner.set_volume(self.volume) {
            error!("Failed to set the initial volume: {e}");
            (false, 1.) // 1 is the default volume of the player
        } else {
            (false, self.volume)
        };
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
    playlist: AlcVec<SongId>,
    #[serde(default)]
    position: Option<Duration>,
    #[serde(default)]
    intercept: Option<PlaylistInfo>,
}

impl Default for PlayerDataLoad {
    fn default() -> Self {
        Self {
            mute: false,
            volume: default_volume(),
            current: None,
            playlist: AlcVec::new(),
            position: None,
            intercept: None,
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
    playlist: &'a AlcVec<SongId>,
    position: Option<Duration>,
    intercept: Option<&'a PlaylistInfo>,
}

/// returns the default volume
fn default_volume() -> f32 {
    1.
}
