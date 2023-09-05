use std::{
    cell::Cell,
    fs::{create_dir_all, File},
    path::Path,
    sync::Arc,
    time::Duration,
};

use log::{error, info, warn};
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    app::UampApp,
    config::Config,
    core::{
        msg::{ComMsg, Msg},
        Result,
    },
    gen_struct,
    library::{Library, SongId},
};

use super::{
    msg::Message, playback::Playback, playlist::Playlist,
    sink_wrapper::SinkWrapper, TimeStamp,
};

gen_struct! {
    pub Player {
        // Reference
        playlist: Playlist { pub, pri },
        ; // value
        current: Option<usize> { pub, pri },
        volume: f32 { pub, pri },
        mute: bool { pub, pri },
        ; // other
        state: Playback,
        inner: SinkWrapper,
    }
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
        lib: &Library,
        index: Option<usize>,
        play: bool,
    ) {
        match index {
            Some(i) if i < self.playlist().len() => self.load(lib, i, play),
            _ => self.stop(),
        }
    }

    /// Toggles the states play/pause, also plays if the state is Stopped and
    /// current is [`Some`]
    pub fn play_pause(&mut self, lib: &Library, play: bool) {
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

    /// Plays the `n`th next song in the playlist
    pub fn play_next(&mut self, lib: &Library, n: usize) {
        self.jump_to(lib, self.current().map(|i| i + n))
    }

    /// Plays the `n`th previous song in the playlist
    pub fn play_prev(&mut self, lib: &Library, n: usize) {
        self.jump_to(lib, self.current().and_then(|i| i.checked_sub(n)))
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
        sender: Arc<UnboundedSender<Msg>>,
        conf: &Config,
    ) -> Self {
        if let Some(p) = conf.player_path() {
            Self::from_json(sender, p)
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
    pub fn to_default_json(&self, conf: &Config) -> Result<()> {
        if !self.change.get() {
            return Ok(());
        }
        if let Some(p) = conf.player_path() {
            self.to_json(p)?;
        }
        self.change.set(false);
        Ok(())
    }

    /// Loads the playback state from the given json file, returns default
    /// [`Player`] on fail
    pub fn from_json(
        sender: Arc<UnboundedSender<Msg>>,
        path: impl AsRef<Path>,
    ) -> Self {
        let data = if let Ok(file) = File::open(path.as_ref()) {
            serde_json::from_reader(file).unwrap_or_default()
        } else {
            info!("player file {:?} doesn't exist", path.as_ref());
            PlayerDataLoad::default()
        };

        let mut res = Self {
            inner: SinkWrapper::new(),
            state: Playback::Stopped,
            playlist: data.playlist,
            current: data.current,
            volume: data.volume,
            mute: data.mute,
            change: Cell::new(false),
        };

        res.init_inner(sender);
        res
    }

    /// Sets the fade duration for play/pause
    pub fn fade_play_pause(&mut self, secs: f32) {
        if let Err(e) = self.inner.fade_play_pause(secs.abs()) {
            error!("Failed to set the fade duration: {e}");
        }
    }

    /// Configures the player
    pub fn load_config(&mut self, conf: &Config) {
        self.fade_play_pause(conf.fade_play_pause());
        self.inner.set_gapless(conf.gapless());
    }

    /// Gets timestamp of the current playback, returns [`None`] if nothing
    /// is playing
    pub fn timestamp(&self) -> Option<TimeStamp> {
        self.inner.get_timestamp().ok()
    }

    /// Seeks to the given position
    pub fn seek_to(&mut self, t: Duration) {
        if let Err(e) = self.inner.seek_to(t) {
            error!("Failed to seek: {e}");
        }
    }
}

impl UampApp {
    /// Handles player event messages
    pub fn player_event(&mut self, msg: Message) -> ComMsg {
        match msg {
            Message::SongEnd => {
                self.player.play_next(&self.library, 1);
            }
        }
        ComMsg::none()
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl Player {
    /// Loads a song into the player.
    /// doesn't check that the index is valid
    fn load(&mut self, lib: &Library, index: usize, play: bool) {
        self.current_set(Some(index));

        match self.inner.load(lib, self.playlist()[index], play) {
            Ok(_) => self.state = Playback::play(play),
            Err(e) => error!("Failed to load song: {e}"),
        }
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

    /// Creates new player from the sender
    fn new(sender: Arc<UnboundedSender<Msg>>) -> Self {
        let mut res = Self {
            playlist: [][..].into(),
            current: None,
            volume: default_volume(),
            mute: false,
            state: Playback::Stopped,
            inner: SinkWrapper::new(),
            change: Cell::new(true),
        };

        res.init_inner(sender);
        res
    }

    fn song_end_handler(sender: &Arc<UnboundedSender<Msg>>) {
        if let Err(e) = sender.send(Msg::Player(Message::SongEnd)) {
            error!("Failed to inform song end: {e}");
        }
    }

    fn init_inner(&mut self, sender: Arc<UnboundedSender<Msg>>) {
        if let Err(e) = self
            .inner
            .on_song_end(move || Self::song_end_handler(&sender))
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
        } else {
            if let Err(e) = self.inner.set_volume(self.volume) {
                error!("Failed to set the initial volume: {e}");
                (false, 1.) // 1 is the default volume of the player
            } else {
                (false, self.volume)
            }
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
