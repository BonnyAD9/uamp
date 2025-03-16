use std::{
    fmt::{Display, Write},
    mem,
    str::FromStr,
    time::{Duration, Instant},
};

use log::info;
use pareg::{
    ArgErrCtx, ArgError, FromArg, FromArgStr, has_any_key, mval_arg, val_arg,
};
use serde::{Deserialize, Serialize};

use crate::{
    env::AppCtrl,
    ext::{Wrap, duration_to_string},
};

use super::{
    Error, Msg, Result, UampApp, library::LoadOpts, player::AddPolicy,
    query::SongOrder,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Simple messages that can be safely send across threads and copied
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ControlMsg {
    /// Toggle/set between play/pause, [`None`] to toggle, [`Some`] to set
    PlayPause(Option<bool>),
    /// Jump to the Nth next song
    NextSong(usize),
    /// Jump to the Nth previous song
    PrevSong(Option<usize>),
    /// Set the volume
    SetVolume(f32),
    /// Increase the volume by `vol_jump * .0`
    VolumeUp(Option<f32>),
    /// Decrease the volume by `vol_jump * .0`
    VolumeDown(Option<f32>),
    /// Toggle/set the mute control, [`None`] to toggle, [`Some`] to set
    Mute(Option<bool>),
    /// Shuffle the current playlist
    Shuffle,
    /// Jump to the given index in the playlist
    PlaylistJump(usize),
    /// Exit the app
    Close,
    /// Search for new songs
    LoadNewSongs(LoadOpts),
    /// Seek to the given timesamp
    SeekTo(Duration),
    /// Seeks forward
    FastForward(Option<Duration>),
    /// Seeks backward
    Rewind(Option<Duration>),
    /// Sorts the top playlist.
    SortPlaylist(SongOrder),
    /// Pop the intercepted playlist.
    PopPlaylist,
    /// Flatten the playlist `n` times. `0` means flatten all.
    Flatten(usize),
    /// Set the playlist add policy.
    SetPlaylistAddPolicy(AddPolicy),
    /// Thriggers save
    Save,
}

impl UampApp {
    /// Handles events for [`ControlMsg`].
    pub(in crate::core) fn control_event(
        &mut self,
        ctrl: &mut AppCtrl,
        msg: ControlMsg,
    ) -> Result<Vec<Msg>> {
        match msg {
            ControlMsg::PlayPause(p) => {
                let pp = p.unwrap_or(!self.player.is_playing());
                if pp {
                    self.hard_pause_at = None;
                }
                self.player.play(&mut self.library, pp)?;
            }
            ControlMsg::NextSong(n) => {
                self.player.play_next(&mut self.library, n);
            }
            ControlMsg::PrevSong(n) => {
                if let Some(t) = self.config.previous_timeout() {
                    if n.is_none() {
                        let now = Instant::now();
                        if now - mem::replace(&mut self.last_prev, now) >= t.0
                        {
                            return Ok(vec![Msg::Control(
                                ControlMsg::SeekTo(Duration::ZERO),
                            )]);
                        }
                    }
                }

                self.player.play_prev(&mut self.library, n.unwrap_or(1));
            }
            ControlMsg::Close => {
                let r = self.save_all(true, ctrl).map(|_| vec![]);
                if ctrl.any_task(|t| t.wait_before_exit()) {
                    self.pending_close = true;
                    return r;
                }
                ctrl.exit();
                return r;
            }
            ControlMsg::Shuffle => {
                self.player
                    .playlist_mut()
                    .shuffle(self.config.shuffle_current());
            }
            ControlMsg::SetVolume(v) => {
                self.player.set_volume(v.clamp(0., 1.))
            }
            ControlMsg::VolumeUp(m) => self.player.set_volume(
                (self.player.volume()
                    + m.unwrap_or(self.config.volume_jump()))
                .clamp(0., 1.),
            ),
            ControlMsg::VolumeDown(m) => self.player.set_volume(
                (self.player.volume()
                    - m.unwrap_or(self.config.volume_jump()))
                .clamp(0., 1.),
            ),
            ControlMsg::PlaylistJump(i) => {
                self.player.jump_to(&mut self.library, i);
            }
            ControlMsg::Mute(b) => {
                self.player.set_mute(b.unwrap_or(!self.player.mute()))
            }
            ControlMsg::LoadNewSongs(opts) => {
                match self.library.start_get_new_songs(
                    &self.config,
                    ctrl,
                    opts,
                ) {
                    Err(e) if matches!(e, Error::InvalidOperation(_)) => {
                        info!("Cannot load new songs: {}", e.log())
                    }
                    Err(e) => {
                        return e.prepend("Cannot load new songs.").err();
                    }
                    _ => {}
                }
            }
            ControlMsg::SeekTo(d) => {
                self.player.seek_to(d)?;
            }
            ControlMsg::FastForward(d) => {
                let t = d.unwrap_or(self.config.seek_jump().0);
                self.player
                    .seek_by(t, true)
                    .map_err(|e| e.prepend("Failed to fast forward."))?;
            }
            ControlMsg::Rewind(d) => {
                let t = d.unwrap_or(self.config.seek_jump().0);
                self.player
                    .seek_by(t, false)
                    .map_err(|e| e.prepend("Failed to rewind."))?;
            }
            ControlMsg::SortPlaylist(ord) => {
                self.player.playlist_mut().sort(
                    &self.library,
                    self.config.simple_sorting(),
                    ord,
                );
            }
            ControlMsg::PopPlaylist => {
                self.player.pop_playlist(&mut self.library);
            }
            ControlMsg::Flatten(cnt) => {
                self.player.flatten(cnt);
            }
            ControlMsg::SetPlaylistAddPolicy(policy) => {
                self.player.playlist_mut().add_policy = policy;
            }
            ControlMsg::Save => self.save_all(false, ctrl)?,
        };

        Ok(vec![])
    }
}

impl Display for ControlMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlMsg::PlayPause(None) => f.write_str("pp"),
            ControlMsg::PlayPause(Some(v)) => {
                if *v {
                    f.write_str("pp=play")
                } else {
                    f.write_str("pp=pause")
                }
            }
            ControlMsg::NextSong(v) => write!(f, "ns={v}"),
            ControlMsg::PrevSong(None) => f.write_str("ps"),
            ControlMsg::PrevSong(Some(v)) => write!(f, "ps={v}"),
            ControlMsg::SetVolume(v) => write!(f, "v={v}"),
            ControlMsg::VolumeUp(None) => f.write_str("vu"),
            ControlMsg::VolumeUp(Some(v)) => write!(f, "vu={v}"),
            ControlMsg::VolumeDown(None) => f.write_str("vd"),
            ControlMsg::VolumeDown(Some(v)) => write!(f, "vd={v}"),
            ControlMsg::Mute(None) => f.write_str("mute"),
            ControlMsg::Mute(Some(v)) => write!(f, "mute={v}"),
            ControlMsg::Shuffle => f.write_str("shuffle"),
            ControlMsg::PlaylistJump(v) => write!(f, "pj={v}"),
            ControlMsg::Close => f.write_char('x'),
            ControlMsg::LoadNewSongs(o) => {
                let s = o.to_string();
                if s.is_empty() {
                    f.write_str("load-songs")
                } else {
                    write!(f, "load-songs={s}")
                }
            }
            ControlMsg::SeekTo(d) => {
                write!(f, "st={}", duration_to_string(*d, false))
            }
            ControlMsg::FastForward(None) => f.write_str("ff"),
            ControlMsg::FastForward(Some(d)) => {
                write!(f, "ff={}", duration_to_string(*d, false))
            }
            ControlMsg::Rewind(None) => f.write_str("rw"),
            ControlMsg::Rewind(Some(d)) => {
                write!(f, "rw={}", duration_to_string(*d, false))
            }
            ControlMsg::SortPlaylist(ord) => write!(f, "sort={ord}"),
            ControlMsg::PopPlaylist => f.write_str("pop"),
            ControlMsg::Flatten(1) => f.write_str("flat"),
            ControlMsg::Flatten(c) => write!(f, "flat={c}"),
            ControlMsg::SetPlaylistAddPolicy(AddPolicy::None) => {
                f.write_str("pap")
            }
            ControlMsg::SetPlaylistAddPolicy(p) => write!(f, "pap={p}"),
            ControlMsg::Save => f.write_str("save"),
        }
    }
}

impl FromStr for ControlMsg {
    type Err = ArgError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            v if has_any_key!(v, '=', "play-pause", "pp") => {
                Ok(ControlMsg::PlayPause(
                    mval_arg::<PlayPause>(v, '=')?.map(|i| i.into()),
                ))
            }
            v if has_any_key!(v, '=', "volume-up", "vol-up", "vu") => {
                Ok(ControlMsg::VolumeUp(mval_arg(v, '=')?))
            }
            v if has_any_key!(v, '=', "volume-down", "vol-down", "vd") => {
                Ok(ControlMsg::VolumeDown(mval_arg(v, '=')?))
            }
            v if has_any_key!(v, '=', "next-song", "ns") => {
                Ok(ControlMsg::NextSong(mval_arg(v, '=')?.unwrap_or(1)))
            }
            v if has_any_key!(v, '=', "previous-song", "ps") => {
                Ok(ControlMsg::PrevSong(mval_arg(v, '=')?))
            }
            v if has_any_key!(v, '=', "playlist-jump", "pj") => {
                Ok(ControlMsg::PlaylistJump(
                    mval_arg(v, '=')?.unwrap_or_default(),
                ))
            }
            v if has_any_key!(v, '=', "volume", "vol", "v") => {
                let vol = val_arg(v, '=')?;
                if !(0.0..=1.).contains(&vol) {
                    return ArgError::parse_msg(
                        "Invalid volume.",
                        v.to_string(),
                    )
                    .inline_msg("Value must be in range from 0 to 1.")
                    .spanned(v.find('=').unwrap_or_default()..v.len())
                    .err();
                }
                Ok(ControlMsg::SetVolume(vol))
            }
            v if has_any_key!(v, '=', "mute") => {
                Ok(ControlMsg::Mute(mval_arg(v, '=')?))
            }
            v if has_any_key!(v, '=', "load-songs") => {
                Ok(ControlMsg::LoadNewSongs(
                    mval_arg(v, '=')?.unwrap_or_default(),
                ))
            }
            "shuffle-playlist" | "shuffle" => Ok(ControlMsg::Shuffle),
            "exit" | "close" | "x" => Ok(ControlMsg::Close),
            v if has_any_key!(v, '=', "seek-to", "seek") => {
                Ok(ControlMsg::SeekTo(val_arg::<Wrap<Duration>>(v, '=')?.0))
            }
            v if has_any_key!(v, '=', "fast-forward", "ff") => {
                Ok(ControlMsg::FastForward(
                    mval_arg::<Wrap<Duration>>(v, '=')?.map(|a| a.0),
                ))
            }
            v if has_any_key!(v, '=', "rewind", "rw") => {
                Ok(ControlMsg::Rewind(
                    mval_arg::<Wrap<Duration>>(v, '=')?.map(|a| a.0),
                ))
            }
            v if has_any_key!(v, '=', "sort-playlist", "sort") => {
                Ok(ControlMsg::SortPlaylist(val_arg(v, '=')?))
            }
            "pop" | "pop-playlist" => Ok(ControlMsg::PopPlaylist),
            v if has_any_key!(v, '=', "flatten", "flat") => {
                Ok(ControlMsg::Flatten(mval_arg(v, '=')?.unwrap_or(1)))
            }
            v if has_any_key!(
                v,
                '=',
                "playlist-add-policy",
                "add-policy",
                "pap"
            ) =>
            {
                Ok(ControlMsg::SetPlaylistAddPolicy(
                    mval_arg(v, '=')?.unwrap_or_default(),
                ))
            }
            "save" => Ok(ControlMsg::Save),
            v => ArgError::UnknownArgument(Box::new(ArgErrCtx::from_msg(
                "Unknown control message.",
                v.to_string(),
            )))
            .err(),
        }
    }
}

impl FromArgStr for ControlMsg {}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

#[derive(FromArg)]
enum PlayPause {
    Play,
    Pause,
}

impl From<PlayPause> for bool {
    fn from(value: PlayPause) -> Self {
        matches!(value, PlayPause::Play)
    }
}
