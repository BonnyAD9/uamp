use std::{
    fmt::{Display, Write},
    mem,
    str::FromStr,
    time::{Duration, Instant},
};

use log::{error, info};
use pareg::{key_mval_arg, key_val_arg, proc::FromArg, ArgError, FromArgStr};
use serde::{Deserialize, Serialize};

use crate::{
    env::AppCtrl,
    ext::{duration_to_string, Wrap},
    starts,
};

use super::{
    library::{Filter, LoadOpts},
    Error, Msg, SongOrder, TaskType, UampApp,
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
    /// Sets the current playlist
    SetPlaylist(Filter),
    /// Pushes new playlist.
    PushPlaylist(Filter),
    /// Sorts the top playlist.
    SortPlaylist(SongOrder),
    /// Pop the intercepted playlist
    PopPlaylist,
    /// Thriggers save
    Save,
}

impl UampApp {
    /// Handles events for [`ControlMsg`].
    pub(in crate::core) fn control_event(
        &mut self,
        ctrl: &mut AppCtrl,
        msg: ControlMsg,
    ) -> Option<Msg> {
        match msg {
            ControlMsg::PlayPause(p) => {
                let pp = p.unwrap_or(!self.player.is_playing());
                if pp {
                    self.hard_pause_at = None;
                }
                self.player.play(&mut self.library, pp);
                return None;
            }
            ControlMsg::NextSong(n) => {
                self.player.play_next(&mut self.library, n);
                return None;
            }
            ControlMsg::PrevSong(n) => {
                if let Some(t) = self.config.previous_timeout() {
                    if n.is_none() {
                        let now = Instant::now();
                        if now - mem::replace(&mut self.last_prev, now) >= t.0
                        {
                            return Some(Msg::Control(ControlMsg::SeekTo(
                                Duration::ZERO,
                            )));
                        }
                    }
                }

                self.player.play_prev(&mut self.library, n.unwrap_or(1));
                if let Err(e) = self.delete_old_logs() {
                    error!("Failed to remove logs: {e}");
                }
                return None;
            }
            ControlMsg::Close => {
                self.save_all(true, ctrl);
                if ctrl.any_task(|t| t != TaskType::Server) {
                    self.pending_close = true;
                    return None;
                }
                // return ComMsg::Command(window::close());
                ctrl.exit();
                return None;
            }
            ControlMsg::Shuffle => {
                self.player
                    .playlist_mut()
                    .shuffle(self.config.shuffle_current());
                return None;
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
                return None;
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
                        info!("Cannot load new songs: {e}")
                    }
                    Err(e) => error!("Cannot load new songs: {e}"),
                    _ => {}
                }
            }
            ControlMsg::SeekTo(d) => {
                self.player.seek_to(d);
                return None;
            }
            ControlMsg::FastForward(d) => {
                let t = d.unwrap_or(self.config.seek_jump().0);
                self.player.seek_by(t, true);
                return None;
            }
            ControlMsg::Rewind(d) => {
                let t = d.unwrap_or(self.config.seek_jump().0);
                self.player.seek_by(t, false);
                return None;
            }
            ControlMsg::SetPlaylist(filter) => {
                let songs = self.library.filter(filter);
                self.player.play_playlist(
                    &mut self.library,
                    songs.into(),
                    false,
                );
            }
            ControlMsg::PushPlaylist(filter) => {
                let songs = self.library.filter(filter);
                self.player.push_playlist(
                    &mut self.library,
                    songs.into(),
                    false,
                );
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
            ControlMsg::Save => self.save_all(false, ctrl),
        };

        None
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
            ControlMsg::SetPlaylist(Filter::All) => f.write_str("sp"),
            ControlMsg::PushPlaylist(Filter::All) => f.write_str("push"),
            ControlMsg::SortPlaylist(ord) => write!(f, "sort={ord}"),
            ControlMsg::PopPlaylist => f.write_str("pop"),
            ControlMsg::Save => f.write_str("save"),
        }
    }
}

impl FromStr for ControlMsg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            v if starts!(v, "play-pause" | "pp") => Ok(ControlMsg::PlayPause(
                key_mval_arg::<&str, PlayPause>(v, '=')?.1.map(|i| i.into()),
            )),
            v if starts!(v, "volume-up" | "vol-up" | "vu") => {
                Ok(ControlMsg::VolumeUp(key_mval_arg::<&str, _>(v, '=')?.1))
            }
            v if starts!(v, "volume-down" | "vol-down" | "vd") => {
                Ok(ControlMsg::VolumeDown(key_mval_arg::<&str, _>(v, '=')?.1))
            }
            v if starts!(v, "next-song" | "ns") => Ok(ControlMsg::NextSong(
                key_mval_arg::<&str, _>(v, '=')?.1.unwrap_or(1),
            )),
            v if starts!(v, "previous-song" | "ps") => {
                Ok(ControlMsg::PrevSong(key_mval_arg::<&str, _>(v, '=')?.1))
            }
            v if starts!(v, "playlist-jump" | "pj") => {
                Ok(ControlMsg::PlaylistJump(
                    key_mval_arg::<&str, _>(v, '=')?.1.unwrap_or_default(),
                ))
            }
            v if starts!(v, "volume" | "vol" | "v") => {
                let v = key_val_arg::<&str, f32>(v, '=')?.1;
                if !(0.0..=1.).contains(&v) {
                    return Err(Error::InvalidValue(
                        "volume must be in range from 0 to 1",
                    ));
                }
                Ok(ControlMsg::SetVolume(v))
            }
            v if starts!(v, "mute") => {
                Ok(ControlMsg::Mute(key_mval_arg::<&str, _>(v, '=')?.1))
            }
            v if starts!(v, "load-songs") => Ok(ControlMsg::LoadNewSongs(
                key_mval_arg::<&str, _>(v, '=')?.1.unwrap_or_default(),
            )),
            "shuffle-playlist" | "shuffle" => Ok(ControlMsg::Shuffle),
            "exit" | "close" | "x" => Ok(ControlMsg::Close),
            v if starts!(v, "seek-to" | "seek") => Ok(ControlMsg::SeekTo(
                key_val_arg::<&str, Wrap<Duration>>(v, '=')?.1 .0,
            )),
            v if starts!(v, "fast-forward" | "ff") => {
                Ok(ControlMsg::FastForward(
                    key_mval_arg::<&str, Wrap<Duration>>(v, '=')?
                        .1
                        .map(|a| a.0),
                ))
            }
            v if starts!(v, "rewind" | "rw") => Ok(ControlMsg::Rewind(
                key_mval_arg::<&str, Wrap<Duration>>(v, '=')?.1.map(|a| a.0),
            )),
            v if starts!(v, "set-playlist" | "sp") => {
                Ok(ControlMsg::SetPlaylist(
                    key_mval_arg::<&str, _>(v, '=')?.1.unwrap_or_default(),
                ))
            }
            v if starts!(v, "push-playlist" | "push") => {
                Ok(ControlMsg::PushPlaylist(
                    key_mval_arg::<&str, _>(v, '=')?.1.unwrap_or_default(),
                ))
            }
            v if starts!(v, "sort-playlist" | "sort") => {
                Ok(ControlMsg::SortPlaylist(key_val_arg::<&str, _>(v, '=')?.1))
            }
            "pop" | "pop-playlist" => Ok(ControlMsg::PopPlaylist),
            "save" => Ok(ControlMsg::Save),
            v => Err(Error::ArgParse(ArgError::UnknownArgument(
                v.to_owned().into(),
            ))),
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
