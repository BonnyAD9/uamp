use std::{cell::Cell, mem, ops::Range, time::Duration};

use bitflags::bitflags;
use itertools::Itertools;
use log::{error, info, warn};
use raplay::{CallbackInfo, Timestamp};
use uamp_proc::TrackChange;

use crate::{
    core::{
        Alias, DataControlMsg, Error, Msg, Result, RtAndle,
        config::Config,
        library::{Library, SongId},
        log_err,
        plugin::DecoderPlugin,
        server::sub,
    },
    ext::Alc,
};

use super::{
    AddPolicy, PlayerMsg, Playlist, playback::Playback,
    sink_wrapper::SinkWrapper,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

#[derive(Debug, TrackChange)]
pub struct Player {
    // Reference
    #[track_ref(pub, pub)]
    playlist: Playlist,
    #[track_ref(pub, pub)]
    playlist_stack: Vec<Playlist>,

    #[track_value(pub, eq)]
    volume: f32,
    #[track_value(pub, eq)]
    mute: bool,

    state: Playback,
    inner: SinkWrapper,
    flags: PlayerFlags,

    #[tracker(Cell::set)]
    change: Cell<bool>,
}

impl Player {
    pub fn add_decoder_plugin(&mut self, plugin: DecoderPlugin) {
        self.inner.add_decoder_plugin(plugin);
    }

    pub fn change(&self) {
        self.change.set(true);
    }

    pub(super) fn reset_change(&self) {
        self.set_change(false);
    }

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
    pub fn change_volume(&mut self, volume: f32) {
        self.set_volume(volume);
        if !self.mute() {
            self.inner.set_volume(volume);
        }
    }

    /// Mute/Unmute.
    ///
    /// - `mute`: when `true` the audio will be muted. This doesn't affect the
    ///   field `volume`.
    pub fn enable_mute(&mut self, mute: bool) {
        let vol = if mute { 0. } else { self.volume() };
        self.set_mute(mute);
        self.inner.set_volume(vol);
    }

    /// Loads the given playlist.
    pub fn play_playlist(
        &mut self,
        lib: &mut Library,
        playlist: Playlist,
        play: bool,
    ) {
        *self.mut_playlist() = playlist;
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
        let id = self.mut_playlist().nth_next(n);
        if id.is_none() {
            self.flags |= PlayerFlags::PLAYLIST_END;
        }
        self.try_load_state(lib, id, n == 1);
    }

    /// Plays the `n`th previous song in the playlist
    pub fn play_prev(&mut self, lib: &mut Library, n: usize) {
        let id = self.mut_playlist().nth_prev(n);
        self.try_load_state(lib, id, false);
    }

    /// Jumps to the given index in the playlist if set.
    pub fn jump_to(&mut self, lib: &mut Library, index: usize) {
        let pf = self
            .playlist()
            .current_idx()
            .map(|i| i + 1 == index)
            .unwrap_or_default();
        let id = self.mut_playlist().jump_to(index);
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
            self.mut_playlist().set_play_pos(t.current)
        }

        let old = mem::replace(self.mut_playlist(), playlist);
        self.mut_playlist_stack().push(old);
        self.try_load(lib, self.playlist.current(), play, false);
    }

    /// Pushes new playlist to the stack without changing the play staty by
    /// moving the now playing song to the start of the new playlist.
    pub fn push_with_cur(&mut self, mut songs: Alc<Vec<SongId>>) {
        songs.splice(0..0, self.mut_playlist().pop_current());
        let old = mem::replace(self.mut_playlist(), songs.into());
        self.mut_playlist_stack().push(old);
        self.inner.unprefetch();
        self.inner.do_prefetch_notify(true);
    }

    /// If there are more playlists in the stack, remove the n on top, but
    /// leave at least one. If `n` is 0 leave only the last one.
    pub fn pop_playlist(&mut self, lib: &mut Library, n: usize) {
        let len = self.playlist_stack.len();
        self.playlist_stack
            .splice(len - n.wrapping_sub(1).min(len.saturating_sub(1)).., []);

        let Some(playlist) = self.playlist_stack.pop() else {
            return;
        };
        *self.mut_playlist() = playlist;
        if self.try_load(
            lib,
            self.playlist.current(),
            self.is_playing(),
            false,
        ) {
            self.mut_playlist().pop_play_pos().map(|p| self.seek_to(p));
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

            let pl = mem::take(self.mut_playlist());
            p.flatten(pl);
            *self.mut_playlist() = p;
        }

        if reprefetch {
            self.inner.do_prefetch_notify(true);
        }
    }

    /// Sets the fade duration for play/pause
    pub fn fade_play_pause(&mut self, t: Duration) {
        self.inner.fade_play_pause(t);
    }

    pub fn gapless(&mut self, enable: bool) {
        self.inner.set_gapless(enable);
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
            warn!("Failed to hard pause: {e:-}");
        }
        // TODO: unload song if stopped
    }

    /// Gets the IDS of the songs in the playlists.
    pub fn get_ids(&mut self) -> Vec<Alc<Vec<SongId>>> {
        Some(self.playlist.clone_songs())
            .into_iter()
            .chain(self.playlist_stack.iter_mut().map(|a| a.clone_songs()))
            .collect()
    }

    /// Retain only songs that match the predicate.
    ///
    /// Returns true if the current song has changed.
    pub fn retain_all(
        &mut self,
        lib: &mut Library,
        mut f: impl FnMut(&Library, SongId, usize) -> bool,
    ) -> bool {
        for p in self.mut_playlist_stack() {
            p.retain(|s, i| f(lib, s, i));
        }
        self.retain_current(lib, &mut f)
    }

    /// Returns true if the current song has changed.
    pub fn retain(
        &mut self,
        lib: &mut Library,
        playlist: usize,
        mut f: impl FnMut(&Library, SongId, usize) -> bool,
    ) -> Result<bool> {
        if playlist == 0 {
            return Ok(self.retain_current(lib, f));
        }
        let pl = self.get_playlist_mut(playlist).ok_or_else(|| {
            Error::invalid_operation()
                .msg(format!("Invalid playlist index `{playlist}`."))
        })?;

        pl.retain(|s, i| f(lib, s, i));

        Ok(false)
    }

    /// Returns true if the current song has changed.
    pub fn retain_current(
        &mut self,
        lib: &mut Library,
        mut f: impl FnMut(&Library, SongId, usize) -> bool,
    ) -> bool {
        let pl = self.mut_playlist();

        let cur = pl.current();
        let next = pl.nth_next(1);
        pl.retain(|s, i| f(lib, s, i));
        let new_cur = pl.current();
        let new_next = pl.nth_next(1);

        if new_cur == cur {
            if new_next != next {
                self.inner.unprefetch();
                self.inner.do_prefetch_notify(true);
            }
            return false;
        }

        if new_cur != next {
            self.inner.unprefetch();
        }

        self.try_load_state(lib, new_cur, true);

        if new_cur.is_none() && cur.is_some() {
            self.flags |= PlayerFlags::PLAYLIST_END;
        }

        true
    }

    /// If any playlist action is triggered, return its messages and clear the
    /// trigger flag.
    pub fn get_playlist_action(
        &mut self,
        default: Option<&Alias>,
    ) -> Option<Msg> {
        if self.flags.contains(PlayerFlags::PLAYLIST_END) {
            self.flags.remove(PlayerFlags::PLAYLIST_END);
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
        self.mut_playlist().add_songs(songs(), policy);
        for p in self.mut_playlist_stack() {
            p.add_songs(songs(), policy);
        }
    }

    /// Reorders playlist stack.
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

        let mut playlists = std::mem::take(self.mut_playlist_stack());
        if let Some(t) = self.timestamp() {
            self.mut_playlist().set_play_pos(t.current)
        }
        playlists.push(mem::replace(self.mut_playlist(), vec![].into()));

        *self.mut_playlist_stack() = order_vec
            .into_iter()
            .map(|a| std::mem::take(&mut playlists[a]))
            .collect();

        self.pop_playlist(lib, 1);

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

    pub fn get_playlist_mut(&mut self, idx: usize) -> Option<&mut Playlist> {
        if idx == 0 {
            return Some(&mut self.playlist);
        }

        let len = self.playlist_stack().len();
        if idx > len {
            return None;
        }

        Some(&mut self.mut_playlist_stack()[len - idx])
    }

    /// Removes songs with indexes in the given ranges. If the currently
    /// playing song changes, returns true.
    ///
    /// Ranges must be sorted by start.
    pub fn remove_ranges(
        &mut self,
        lib: &mut Library,
        playlist: usize,
        ranges: impl IntoIterator<Item = Range<usize>>,
    ) -> Result<bool> {
        let mut ranges = ranges.into_iter();
        let mut range = ranges.next();

        self.retain(lib, playlist, |_, _, i| {
            let Some(mut r) = range.clone() else {
                return true;
            };

            while i >= r.end {
                range = ranges.next();
                let Some(rn) = range.clone() else {
                    return true;
                };
                r = rn;
            }

            if i < r.start {
                return true;
            }

            !r.contains(&i)
        })
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
            flags: PlayerFlags::NONE,
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
            flags: PlayerFlags::NONE,
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

    /// Move to the next song without loading it because it was successfully
    /// prefetched.
    pub(super) fn prefetch_success(&mut self) {
        self.mut_playlist().nth_next(1);
    }

    /// Prefetch the next song if available.
    pub(super) fn prefetch(&mut self, lib: &mut Library) {
        let Some(id) = self.playlist().peek() else {
            return;
        };

        if let Err(e) = self.inner.prefetch(lib, id) {
            error!("Failed to prefetch song {:?}: {e:-}", lib[id].path(),);
        }
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

bitflags! {
    #[derive(Debug, Clone, Copy)]
    struct PlayerFlags: u32 {
        const NONE = 0x0;
        const PLAYLIST_END = 0x1;
    }
}

impl Player {
    /// Load with the current playback state. If pf is true, it means that the
    /// prefeched song can be used.
    fn try_load_state(
        &mut self,
        lib: &mut Library,
        id: Option<SongId>,
        pf: bool,
    ) -> bool {
        let Some(id) = id else {
            if !self.state.is_stopped() {
                self.inner.play(false);
                log_err(
                    "Failed to hard pause the playback when stopped.",
                    self.inner.hard_pause(),
                );
                self.state = Playback::Stopped;
            }
            return false;
        };

        match self.state {
            Playback::Stopped => true,
            _ => self.load(lib, id, self.is_playing(), pf),
        }
    }

    /// Load with the given playback state. If pf is true, it means that the
    /// prefeched song can be used.
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

    /// If pf is true, it means that the prefeched song can be used.
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
                error!("Failed to load song {:?}: {e:-}", lib[id].path(),);
                info!("Moving to the next song.");
                let next = self.playlist.nth_next(1);
                if next.is_none() {
                    self.flags |= PlayerFlags::PLAYLIST_END;
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
