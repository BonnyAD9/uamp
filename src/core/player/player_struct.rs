use std::{cell::Cell, mem, time::Duration};

use itertools::Itertools;
use log::{error, info, warn};
use raplay::{CallbackInfo, Timestamp};

use crate::{
    core::{
        Alias, DataControlMsg, Error, Msg, Result, RtAndle,
        config::Config,
        library::{Library, SongId},
        server::sub,
    },
    ext::AlcVec,
    gen_struct,
};

use super::{
    AddPolicy, PlayerMsg, Playlist, playback::Playback,
    sink_wrapper::SinkWrapper,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

gen_struct! {
    #[derive(Debug)]
    pub Player {
        // Reference
        playlist: Playlist { pub, pub },
        playlist_stack: Vec<Playlist> { pub, pub },
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
            self.load(lib, id, play, false);
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
        self.try_load(lib, self.playlist().current(), play, false);
    }

    /// Returns true if the state is [`Playback::Playing`]
    pub fn is_playing(&self) -> bool {
        matches!(self.state, Playback::Playing)
    }

    pub fn playback_state(&self) -> Playback {
        self.state
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
        self.try_load_state(lib, id, n == 1);
    }

    /// Plays the `n`th previous song in the playlist
    pub fn play_prev(&mut self, lib: &mut Library, n: usize) {
        let id = self.playlist_mut().nth_prev(n);
        self.try_load_state(lib, id, false);
    }

    /// Jumps to the given index in the playlist if set.
    pub fn jump_to(&mut self, lib: &mut Library, index: usize) {
        let pf = self
            .playlist()
            .current_idx()
            .map(|i| i + 1 == index)
            .unwrap_or_default();
        let id = self.playlist_mut().jump_to(index);
        self.try_load_state(lib, id, pf);
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
        self.try_load(lib, self.playlist.current(), play, false);
    }

    /// Pushes new playlist to the stack without changing the play staty by
    /// moving the now playing song to the start of the new playlist.
    pub fn push_with_cur(&mut self, mut songs: AlcVec<SongId>) {
        songs.splice(0..0, self.playlist_mut().pop_current());
        let old = mem::replace(self.playlist_mut(), songs.into());
        self.playlist_stack_mut().push(old);
        self.inner.unprefetch();
        self.inner.do_prefetch_notify(true);
    }

    /// If there are more playlists in the stack, end the top one.
    pub fn pop_playlist(&mut self, lib: &mut Library) {
        let Some(playlist) = self.playlist_stack.pop() else {
            return;
        };
        *self.playlist_mut() = playlist;
        if self.try_load(
            lib,
            self.playlist.current(),
            self.is_playing(),
            false,
        ) {
            self.playlist_mut().pop_play_pos().map(|p| self.seek_to(p));
        }
    }

    /// Insert the current playlist onto the next top playlist `cnt` times.
    /// This is seamless because the currently playing song stays the same.
    pub fn flatten(&mut self, cnt: usize) {
        let cnt = if cnt == 0 { usize::MAX } else { cnt };
        let reprefetch = self
            .playlist()
            .current_idx()
            .map(|i| {
                i + 1 == self.playlist().len()
                    && !self.playlist_stack().is_empty()
            })
            .unwrap();
        for _ in 0..cnt {
            let Some(mut p) = self.playlist_stack.pop() else {
                break;
            };

            let pl = mem::take(self.playlist_mut());
            p.flatten(pl);
            *self.playlist_mut() = p;
        }

        if reprefetch {
            self.inner.do_prefetch_notify(true);
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
            self.inner.get_timestamp()
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
        // TODO: unload song if stopped
    }

    /// Gets the IDS of the songs in the playlists.
    pub fn get_ids(&mut self) -> Vec<AlcVec<SongId>> {
        Some(self.playlist.clone_songs())
            .into_iter()
            .chain(self.playlist_stack.iter_mut().map(|a| a.clone_songs()))
            .collect()
    }

    /// Retain only songs that match the predicate.
    pub fn retain(&mut self, mut f: impl FnMut(&SongId) -> bool) {
        self.playlist_mut().retain(&mut f);
        for p in self.playlist_stack_mut() {
            p.retain(&mut f)
        }
        self.inner.unprefetch();
        self.inner.do_prefetch_notify(true);
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

    /// Add new songs to the playlists based on the given default policy or
    /// their policy.
    pub fn add_songs<I: Iterator<Item = SongId>>(
        &mut self,
        mut songs: impl FnMut() -> I,
        policy: Option<AddPolicy>,
    ) {
        self.playlist_mut().add_songs(songs(), policy);
        for p in self.playlist_stack_mut() {
            p.add_songs(songs(), policy);
        }
    }

    pub fn reorder_playlist(
        &mut self,
        lib: &mut Library,
        order: &[usize],
    ) -> Result<()> {
        if order.is_empty() {
            return Ok(());
        }

        let stack_len = self.playlist_stack().len() + 1;
        if order.len() > stack_len {
            return Error::invalid_operation().msg(format!(
                "Playlist stack has only {stack_len} playlist but the reorder \
                requires {} elements.",
                order.len()
            )).err();
        }

        if let Some(n) = order.iter().find(|a| **a >= stack_len) {
            return Error::invalid_operation()
                .msg(format!(
                    "Reorder contains index out of bounds ({n}/{stack_len})"
                ))
                .err();
        }

        let mut order_vec = vec![];

        for i in 0..stack_len {
            if !order.contains(&(stack_len - i - 1)) {
                order_vec.push(i);
            }
        }

        order_vec.extend(order.iter().rev().map(|i| stack_len - i - 1));

        let mut playlists = std::mem::take(self.playlist_stack_mut());
        if let Some(t) = self.timestamp() {
            self.playlist_mut().set_play_pos(t.current)
        }
        playlists.push(mem::replace(self.playlist_mut(), vec![].into()));

        *self.playlist_stack_mut() = order_vec
            .into_iter()
            .map(|a| std::mem::take(&mut playlists[a]))
            .collect();

        self.pop_playlist(lib);

        Ok(())
    }

    pub fn get_sub(&mut self) -> sub::Player {
        // These operations doesn't actually mutate the data, just the way they
        // are stored (AlcVec).
        sub::Player {
            playlist: self.sub_playlist(),
            playlist_stack: self
                .playlist_stack
                .iter_mut()
                .map(sub::Playlist::new)
                .collect_vec(),
            volume: self.volume(),
            mute: self.mute(),
            state: self.state,
        }
    }

    pub fn sub_playlist(&mut self) -> sub::Playlist {
        sub::Playlist::new(&mut self.playlist)
    }

    /// Creates new player from the sender
    pub(super) fn new_default(rt: RtAndle) -> Self {
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

        res.init_inner(rt);
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
    pub(super) fn init_inner(&mut self, rt: RtAndle) {
        self.inner
            .on_callback(move |msg| Self::inner_callback_handler(msg, &rt));

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

    /// Move to the next song without loading it because it was successfully
    /// prefetched.
    pub(super) fn prefetch_success(&mut self) {
        self.playlist_mut().nth_next(1);
    }

    /// Prefetch the next song if available.
    pub(super) fn prefetch(&mut self, lib: &mut Library) {
        let Some(id) = self.playlist().peek() else {
            return;
        };

        if let Err(e) = self.inner.prefetch(lib, id) {
            error!(
                "Failed to prefetch song {:?}: {}",
                lib[id].path(),
                e.log()
            );
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
        pf: bool,
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
            _ => self.load(lib, id, self.is_playing(), pf),
        }
    }

    fn try_load(
        &mut self,
        lib: &mut Library,
        id: Option<SongId>,
        play: bool,
        pf: bool,
    ) -> bool {
        if let Some(id) = id {
            self.load(lib, id, play, pf)
        } else {
            self.stop();
            false
        }
    }

    fn load(
        &mut self,
        lib: &mut Library,
        id: SongId,
        play: bool,
        pf: bool,
    ) -> bool {
        let err = if pf {
            self.inner.load_or_prefetched(lib, id, play)
        } else {
            self.inner.load(lib, id, play)
        };

        match err {
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
                self.try_load(lib, next, play, pf)
            }
        }
    }

    fn inner_callback_handler(msg: CallbackInfo, rt: &RtAndle) {
        let message = match msg {
            CallbackInfo::SourceEnded(s) => Msg::Player(PlayerMsg::SongEnd(s)),
            CallbackInfo::PrefetchTime(_) => Msg::Player(PlayerMsg::Prefetch),
            CallbackInfo::PauseEnds(i) => {
                Msg::Player(PlayerMsg::HardPauseAt(i))
            }
            _ => return,
        };

        rt.msg(message);
    }
}

/// returns the default volume
pub(super) fn default_volume() -> f32 {
    1.
}
