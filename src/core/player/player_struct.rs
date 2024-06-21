use std::{cell::Cell, mem, time::Duration};

use futures::channel::mpsc::UnboundedSender;
use log::{error, warn};
use raplay::{CallbackInfo, Timestamp};

use crate::{
    core::{
        config::Config,
        library::{Library, LibraryUpdate, SongId},
        Msg, UampApp,
    },
    ext::alc_vec::AlcVec,
    gen_struct,
};

use super::{
    playback::Playback, sink_wrapper::SinkWrapper, PlayerMsg, Playlist,
};

gen_struct! {
    #[derive(Debug)]
    pub Player {
        // Reference
        playlist: Playlist { pub, pub },
        intercept: Option<Playlist> { pub(super), pri },
        ; // value
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
    pub fn play(&mut self, lib: &mut Library, play: bool) {
        if !self.state.is_stopped() {
            match self.inner.play(play) {
                Ok(_) => {
                    self.state = Playback::play(play);
                }
                Err(e) => error!("Failed to play/pause: {}", e),
            }
            return;
        }

        if let Some(id) = self.playlist().current() {
            self.load(lib, id, play);
        } else {
            _ = self.inner.play(false);
            error!("Cannot play/pause, the playlist is empty.");
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

    /// Loads the given playlist
    pub fn play_playlist(
        &mut self,
        lib: &mut Library,
        playlist: Playlist,
        play: bool,
    ) {
        *self.playlist_mut() = playlist;
        self.try_load(lib, self.playlist().current(), play);
    }

    /// Returns true if the state is [`Playback::Playing`]
    pub fn is_playing(&self) -> bool {
        matches!(self.state, Playback::Playing)
    }

    /// gets the now playing song if available
    pub fn now_playing(&self) -> Option<SongId> {
        self.playlist.current()
    }

    /// Plays the `n`th next song in the playlist
    pub fn play_next(&mut self, lib: &mut Library, n: usize) {
        let id = self.playlist_mut().nth_next(n);
        self.try_load_state(lib, id);
    }

    /// Plays the `n`th previous song in the playlist
    pub fn play_prev(&mut self, lib: &mut Library, n: usize) {
        let id = self.playlist_mut().nth_prev(n);
        self.try_load_state(lib, id);
    }

    /// Jumps to the given index in the playlist if set.
    pub fn jump_to(&mut self, lib: &mut Library, index: usize) {
        let id = self.playlist_mut().jump_to(index);
        self.try_load_state(lib, id);
    }

    /// Changes the state to [`Playback::Stopped`]
    pub fn stop(&mut self) {
        match self.inner.play(false) {
            Ok(_) => self.state = Playback::Stopped,
            Err(e) => error!("Failed to stop playback: {}", e),
        }
    }

    pub fn push_playlist(
        &mut self,
        lib: &mut Library,
        playlist: Playlist,
        play: bool,
    ) {
        if self.intercept.is_some() {
            self.play_playlist(lib, playlist, play);
            return;
        }
        if let Some(t) = self.timestamp() {
            self.playlist.set_play_pos(t.current)
        }

        self.intercept = Some(mem::replace(self.playlist_mut(), playlist));
        self.try_load(lib, self.playlist.current(), play);
    }

    pub fn pop_playlist(&mut self, lib: &mut Library) {
        let Some(playlist) = self.intercept.take() else {
            return;
        };
        *self.playlist_mut() = playlist;
        if self.try_load_state(lib, self.playlist.current()) {
            self.playlist_mut().pop_play_pos().map(|p| self.seek_to(p));
        }
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

    pub fn hard_pause(&mut self) {
        if let Err(e) = self.inner.hard_pause() {
            warn!("Failed to hard pause: {e}");
        }
    }

    pub fn get_ids(&mut self) -> (AlcVec<SongId>, Option<AlcVec<SongId>>) {
        (
            self.playlist.clone_songs(),
            self.intercept.as_mut().map(|p| p.clone_songs()),
        )
    }

    pub fn remove_deleted(&mut self, lib: &Library) {
        self.playlist_mut().remove_deleted(lib);
        if let Some(p) = self.intercept_mut() {
            p.remove_deleted(lib);
        }
    }

    /// Creates new player from the sender
    pub(super) fn new_default(sender: UnboundedSender<Msg>) -> Self {
        let mut res = Self {
            playlist: Playlist::default(),
            intercept: None,
            volume: default_volume(),
            mute: false,
            state: Playback::Stopped,
            inner: SinkWrapper::new(),
            change: Cell::new(true),
        };

        res.init_inner(sender);
        res
    }

    pub(super) fn new(
        inner: SinkWrapper,
        state: Playback,
        playlist: Playlist,
        intercept: Option<Playlist>,
        volume: f32,
        mute: bool,
        change: bool,
    ) -> Self {
        Self {
            inner,
            state,
            playlist,
            intercept,
            volume,
            mute,
            change: change.into(),
        }
    }

    pub(super) fn init_inner(&mut self, sender: UnboundedSender<Msg>) {
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

    pub(super) fn get_change(&self) -> bool {
        self.change.get()
    }

    pub(super) fn set_change(&self, val: bool) {
        self.change.set(val);
    }
}

impl UampApp {
    /// Handles player event messages
    pub fn player_event(&mut self, msg: PlayerMsg) -> Option<Msg> {
        match msg {
            PlayerMsg::SongEnd => {
                self.player.play_next(&mut self.library, 1);
            }
            PlayerMsg::HardPauseAt(i) => self.hard_pause_at = Some(i),
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
    fn try_load_state(
        &mut self,
        lib: &mut Library,
        id: Option<SongId>,
    ) -> bool {
        let Some(id) = id else {
            if !self.state.is_stopped() {
                self.state = Playback::Stopped;
                if let Err(e) = self.inner.play(false) {
                    error!("Failed to stop playback: {e}");
                }
            }
            return false;
        };

        match self.state {
            Playback::Stopped => true,
            _ => self.load(lib, id, self.is_playing()),
        }
    }

    /// Tries to load a song into the library. Stops on failure.
    fn try_load(
        &mut self,
        lib: &mut Library,
        id: Option<SongId>,
        play: bool,
    ) -> bool {
        if let Some(id) = id {
            self.load(lib, id, play)
        } else {
            self.stop();
            false
        }
    }

    /// Loads a song into the player.
    fn load(&mut self, lib: &mut Library, id: SongId, play: bool) -> bool {
        match self.inner.load(lib, id, play) {
            Ok(_) => {
                self.state = Playback::play(play);
                true
            }
            Err(e) => {
                error!("Failed to load song {:?}: {e}", lib[id].path());
                let next = self.playlist.nth_next(1);
                self.try_load(lib, next, play)
            }
        }
    }

    fn song_end_handler(msg: CallbackInfo, sender: &UnboundedSender<Msg>) {
        let message = match msg {
            CallbackInfo::SourceEnded => Msg::Player(PlayerMsg::SongEnd),
            CallbackInfo::PauseEnds(i) => {
                Msg::Player(PlayerMsg::HardPauseAt(i))
            }
            _ => todo!("Fix me at {}:{}::", file!(), line!()),
        };

        if let Err(e) = sender.unbounded_send(message) {
            error!("Failed to sink callback message: {e}");
        }
    }
}

/// returns the default volume
pub(super) fn default_volume() -> f32 {
    1.
}
