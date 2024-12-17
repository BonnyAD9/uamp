use std::{cell::Cell, mem, time::Duration};

use futures::channel::mpsc::UnboundedSender;
use log::{error, info, warn};
use raplay::{CallbackInfo, Timestamp};

use crate::{
    core::{
        config::Config,
        library::{Library, SongId},
        Alias, DataControlMsg, Error, Msg, Result,
    },
    ext::AlcVec,
    gen_struct,
};

use super::{
    playback::Playback, sink_wrapper::SinkWrapper, PlayerMsg, Playlist,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

gen_struct! {
    #[derive(Debug)]
    pub Player {
        // Reference
        playlist: Playlist { pub, pub },
        playlist_stack: Vec<Playlist> { pub, pri },
        ; // value
        volume: f32 { pub, pri },
        mute: bool { pub, pri },
        ; // other
        state: Playback,
        inner: SinkWrapper,
        playlist_ended: bool,
    }
}

impl Player {
    /// Play/Pause.
    ///
    /// - `play`: play when `true`, otherwise pause.
    pub fn play(&mut self, lib: &mut Library, play: bool) -> Result<()> {
        if !self.state.is_stopped() {
            self.inner.play(play);
            self.state = Playback::play(play);
            return Ok(());
        }

        if let Some(id) = self.playlist().current() {
            self.load(lib, id, play);
            Ok(())
        } else {
            self.inner.play(false);
            Error::invalid_operation()
                .msg("Cannot play/pause.")
                .reason("The playlist is empty.")
                .err()
        }
    }

    /// Sets the playback volume.
    ///
    /// - `volume`: Value from 0 to 1 (can be outside of this range but it may
    ///   result in distorted audio). It is on square root scale.
    pub fn set_volume(&mut self, volume: f32) {
        self.volume_set(volume);
        self.inner.set_volume(volume);
    }

    /// Mute/Unmute.
    ///
    /// - `mute`: when `true` the audio will be muted. This doesn't affect the
    ///   field `volume`.
    pub fn set_mute(&mut self, mute: bool) {
        let vol = if mute { 0. } else { self.volume() };
        self.mute_set(mute);
        self.inner.set_volume(vol);
    }

    /// Loads the given playlist.
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
        if id.is_none() {
            self.playlist_ended = true;
        }
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
        if !self.state.is_stopped() {
            self.inner.play(false);
            self.state = Playback::Stopped;
        }
    }

    /// Push the given playlist to the stack and set it as current.
    pub fn push_playlist(
        &mut self,
        lib: &mut Library,
        playlist: Playlist,
        play: bool,
    ) {
        if let Some(t) = self.timestamp() {
            self.playlist_mut().set_play_pos(t.current)
        }

        let old = mem::replace(self.playlist_mut(), playlist);
        self.playlist_stack_mut().push(old);
        self.try_load(lib, self.playlist.current(), play);
    }

    /// Pushes new playlist to the stack without changing the play staty by
    /// moving the now playing song to the start of the new playlist.
    pub fn push_with_cur(&mut self, mut songs: AlcVec<SongId>) {
        songs.splice(0..0, self.playlist_mut().pop_current());
        let old = mem::replace(self.playlist_mut(), songs.into());
        self.playlist_stack_mut().push(old);
    }

    /// If there are more playlists in the stack, end the top one.
    pub fn pop_playlist(&mut self, lib: &mut Library) {
        let Some(playlist) = self.playlist_stack.pop() else {
            return;
        };
        *self.playlist_mut() = playlist;
        if self.try_load_state(lib, self.playlist.current()) {
            self.playlist_mut().pop_play_pos().map(|p| self.seek_to(p));
        }
    }

    /// Insert the current playlist onto the next top playlist `cnt` times.
    /// This is seamless because the currently playing song stays the same.
    pub fn flatten(&mut self, cnt: usize) {
        let cnt = if cnt == 0 { usize::MAX } else { cnt };
        for _ in 0..cnt {
            let Some(mut p) = self.playlist_stack.pop() else {
                break;
            };

            let pl = mem::take(self.playlist_mut());
            p.flatten(pl);
            *self.playlist_mut() = p;
        }
    }

    /// Sets the fade duration for play/pause
    pub fn fade_play_pause(&mut self, t: Duration) {
        self.inner.fade_play_pause(t);
    }

    /// Configures the player
    pub fn load_config(&mut self, conf: &Config) {
        self.fade_play_pause(conf.fade_play_pause().0);
        self.inner.set_gapless(conf.gapless());
    }

    /// Gets timestamp of the current playback, returns [`None`] if nothing
    /// is playing.
    pub fn timestamp(&self) -> Option<Timestamp> {
        if self.state.is_stopped() {
            None
        } else {
            self.inner.get_timestamp().ok()
        }
    }

    /// Seeks to the given position
    pub fn seek_to(&mut self, t: Duration) -> Result<Timestamp> {
        self.inner
            .seek_to(t)
            .map_err(|e| e.prepend("Failed to seek."))
    }

    /// Seeks by the given amount.
    ///
    /// - `t`: seek amount
    /// - `forward`: the direction of seeking. `true` - forward, `false` -
    ///   backwards
    pub fn seek_by(
        &mut self,
        t: Duration,
        forward: bool,
    ) -> Result<Timestamp> {
        self.inner.seek_by(t, forward)
    }

    /// Does hard pause without the fading (if configured).
    pub fn hard_pause(&mut self) {
        if let Err(e) = self.inner.hard_pause() {
            warn!("Failed to hard pause: {}", e.log());
        }
    }

    /// Gets the IDS of the songs in the playlists.
    pub fn get_ids(&mut self) -> Vec<AlcVec<SongId>> {
        Some(self.playlist.clone_songs())
            .into_iter()
            .chain(self.playlist_stack.iter_mut().map(|a| a.clone_songs()))
            .collect()
    }

    /// Removes all deleted songs from the playlists.
    pub fn remove_deleted(&mut self, lib: &Library) {
        self.playlist_mut().remove_deleted(lib);
        for p in self.playlist_stack_mut() {
            p.remove_deleted(lib)
        }
    }

    /// If any playlist action is triggered, return its messages and clear the
    /// trigger flag.
    pub fn get_playlist_action(
        &mut self,
        default: Option<&Alias>,
    ) -> Option<Msg> {
        if self.playlist_ended {
            self.playlist_ended = false;
            self.playlist()
                .on_end
                .clone()
                .or_else(|| default.cloned())
                .map(|a| DataControlMsg::Alias(a).into())
        } else {
            None
        }
    }

    /// Creates new player from the sender
    pub(super) fn new_default(sender: UnboundedSender<Msg>) -> Self {
        let mut res = Self {
            playlist: Playlist::default(),
            playlist_stack: vec![],
            volume: default_volume(),
            mute: false,
            state: Playback::Stopped,
            inner: SinkWrapper::new(),
            change: Cell::new(true),
            playlist_ended: false,
        };

        res.init_inner(sender);
        res
    }

    /// Creates new player from the given configuration. Doesn't init.
    pub(super) fn new(
        inner: SinkWrapper,
        state: Playback,
        playlist: Playlist,
        playlist_stack: Vec<Playlist>,
        volume: f32,
        mute: bool,
        change: bool,
    ) -> Self {
        Self {
            inner,
            state,
            playlist,
            playlist_stack,
            volume,
            mute,
            change: change.into(),
            playlist_ended: false,
        }
    }

    /// Initializes the player.
    pub(super) fn init_inner(&mut self, sender: UnboundedSender<Msg>) {
        self.inner.on_callback(move |msg| {
            Self::inner_callback_handler(msg, &sender)
        });

        if self.mute {
            self.inner.set_volume(0.);
        } else {
            self.inner.set_volume(self.volume);
        }
    }

    /// Gets value indicating whether there was any changle since the last
    /// save.
    pub(super) fn get_change(&self) -> bool {
        self.change.get()
    }

    /// Sets value indicating whether there was any change since the last save.
    pub(super) fn set_change(&self, val: bool) {
        self.change.set(val);
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
                self.inner.play(false);
                self.state = Playback::Stopped;
            }
            return false;
        };

        match self.state {
            Playback::Stopped => true,
            _ => self.load(lib, id, self.is_playing()),
        }
    }

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

    fn load(&mut self, lib: &mut Library, id: SongId, play: bool) -> bool {
        match self.inner.load(lib, id, play) {
            Ok(_) => {
                self.state = Playback::play(play);
                true
            }
            Err(e) => {
                error!(
                    "Failed to load song {:?}: {}",
                    lib[id].path(),
                    e.log()
                );
                info!("Moving to the next song.");
                let next = self.playlist.nth_next(1);
                if next.is_none() {
                    self.playlist_ended = true;
                }
                self.try_load(lib, next, play)
            }
        }
    }

    fn inner_callback_handler(
        msg: CallbackInfo,
        sender: &UnboundedSender<Msg>,
    ) {
        let message = match msg {
            CallbackInfo::SourceEnded => Msg::Player(PlayerMsg::SongEnd),
            CallbackInfo::PauseEnds(i) => {
                Msg::Player(PlayerMsg::HardPauseAt(i))
            }
            _ => todo!("Fix me at {}:{}::", file!(), line!()),
        };

        if let Err(e) = sender.unbounded_send(message) {
            error!("Failed to set callback message: {e}");
        }
    }
}

/// returns the default volume
pub(super) fn default_volume() -> f32 {
    1.
}
