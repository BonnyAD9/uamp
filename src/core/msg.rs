use core::fmt::Debug;
use std::{
    fmt::{Display, Write},
    mem::replace,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use log::{error, info};
use pareg::{key_mval_arg, key_val_arg, proc::FromArg, ArgError, FromArgStr};
use serde::{Deserialize, Serialize};

use crate::{
    core::UampApp,
    env::AppCtrl,
    ext::extensions::{duration_to_string, Wrap},
};

use super::{
    config::ConfigMsg,
    library::{Filter, LoadOpts, SongId},
    player::{PlayerMsg, Playlist},
    Error, SongOrder, TaskType,
};

/// Event messages in uamp
#[allow(missing_debug_implementations)]
#[derive(Clone, Debug, Default)]
pub enum Msg {
    /// Play song song at the given index in the playlist
    PlaySong(PlayMsg),
    /// Some simple messages
    Control(ControlMsg),
    /// More complicated messages
    DataControl(DataControlMsg),
    /// Player messges handled by the player
    Player(PlayerMsg),
    /// Dellegate the message
    Delegate(Arc<dyn MessageDelegate>),
    Config(ConfigMsg),
    // General update
    Tick,
    #[default]
    None,
}

impl Msg {
    pub fn delegate<I, D>(d: I) -> Self
    where
        D: MessageDelegate + 'static,
        I: Into<D>,
    {
        Self::Delegate(Arc::new(d.into()))
    }
}

/// only simple messages that can be safely send across threads and copied
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
/// Messages that can be safely send across threads
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DataControlMsg {
    Alias(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AnyControlMsg {
    Control(ControlMsg),
    Data(DataControlMsg),
}

#[derive(Clone, Debug)]
pub enum PlayMsg {
    Playlist(usize, Arc<Vec<SongId>>),
    TmpPath(Arc<Path>),
}

impl UampApp {
    /// handles the control events
    pub fn control_event(
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
                return Some(Msg::Tick);
            }
            ControlMsg::NextSong(n) => {
                self.player.play_next(&mut self.library, n);
                return Some(Msg::Tick);
            }
            ControlMsg::PrevSong(n) => {
                if let Some(t) = self.config.previous_timeout() {
                    if n.is_none() {
                        let now = Instant::now();
                        if now - replace(&mut self.last_prev, now) >= t.0 {
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
                return Some(Msg::Tick);
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
                return Some(Msg::Tick);
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
                return Some(Msg::Tick);
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
                return Some(Msg::Tick);
            }
            ControlMsg::FastForward(d) => {
                let t = d.unwrap_or(self.config.seek_jump().0);
                self.player.seek_by(t, true);
                return Some(Msg::Tick);
            }
            ControlMsg::Rewind(d) => {
                let t = d.unwrap_or(self.config.seek_jump().0);
                self.player.seek_by(t, false);
                return Some(Msg::Tick);
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

    pub fn data_control_event(
        &mut self,
        ctrl: &mut AppCtrl,
        msg: DataControlMsg,
    ) -> Option<Msg> {
        match msg {
            DataControlMsg::Alias(name) => {
                for m in self.config.control_aliases().get(&name)?.clone() {
                    self.update(ctrl, m.into())
                }

                None
            }
        }
    }

    pub fn play_event(&mut self, msg: PlayMsg) -> Option<Msg> {
        match msg {
            PlayMsg::Playlist(index, songs) => {
                self.player.play_playlist(
                    &mut self.library,
                    Playlist::new(songs, index),
                    true,
                );
            }
            PlayMsg::TmpPath(path) => {
                let id = match self.library.add_tmp_path(path.as_ref()) {
                    Err(e) => {
                        error!("Failed to load song {path:?}: {e}");
                        return None;
                    }
                    Ok(id) => id,
                };

                self.player.push_playlist(
                    &mut self.library,
                    vec![id].into(),
                    true,
                );
            }
        }

        Some(Msg::Tick)
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

pub trait MessageDelegate: Sync + Send + Debug {
    fn update(&self, app: &mut UampApp, ctrl: &mut AppCtrl) -> Option<Msg>;
}

pub struct FnDelegate<T>(T)
where
    T: Sync + Send + Fn(&mut UampApp, &mut AppCtrl) -> Option<Msg>;

impl<T> MessageDelegate for FnDelegate<T>
where
    T: Sync + Send + Fn(&mut UampApp, &mut AppCtrl) -> Option<Msg>,
{
    fn update(&self, app: &mut UampApp, ctrl: &mut AppCtrl) -> Option<Msg> {
        self.0(app, ctrl)
    }
}

impl<T> Debug for FnDelegate<T>
where
    T: Sync + Send + Fn(&mut UampApp, &mut AppCtrl) -> Option<Msg>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FnDelegate").finish()
    }
}

impl<T> From<T> for FnDelegate<T>
where
    T: Sync + Send + Fn(&mut UampApp, &mut AppCtrl) -> Option<Msg>,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}

/// creates expression that checks whether a variable starts with any of the
/// strings
///
/// # Example
/// ```
/// let val = "arg2=hi";
/// if starts!(val, "arg1" | "arg2") {
///     // now we know that `val` starts either with `"arg1"` or `"arg2"`
/// }
/// ```
#[macro_export]
macro_rules! starts {
    ($i:ident, $($s:literal)|+) => {{
        matches!($i, $($s)|+) || $($i.starts_with(concat!($s, "=")))||+
    }};
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

#[derive(Serialize)]
#[serde(untagged)]
enum PlayMsgSe<'a> {
    Playlist(usize, &'a Vec<SongId>),
    TmpPath(&'a Path),
}

impl<'a> From<&'a PlayMsg> for PlayMsgSe<'a> {
    fn from(value: &'a PlayMsg) -> Self {
        match value {
            PlayMsg::Playlist(idx, songs) => Self::Playlist(*idx, songs),
            PlayMsg::TmpPath(path) => Self::TmpPath(path.as_ref()),
        }
    }
}

impl Serialize for PlayMsg {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        PlayMsgSe::from(self).serialize(serializer)
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PlayMsgDe {
    Playlist(usize, Vec<SongId>),
    TmpPath(PathBuf),
}

impl From<PlayMsgDe> for PlayMsg {
    fn from(value: PlayMsgDe) -> Self {
        match value {
            PlayMsgDe::Playlist(idx, songs) => {
                Self::Playlist(idx, Arc::new(songs))
            }
            PlayMsgDe::TmpPath(path) => Self::TmpPath(path.into()),
        }
    }
}

impl<'de> Deserialize<'de> for PlayMsg {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        PlayMsgDe::deserialize(deserializer).map(|m| m.into())
    }
}

impl FromStr for DataControlMsg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            v if starts!(v, "al" | "alias") => {
                Ok(DataControlMsg::Alias(key_val_arg::<&str, _>(v, '=')?.1))
            }
            v => Err(Error::ArgParse(ArgError::UnknownArgument(
                v.to_owned().into(),
            ))),
        }
    }
}

impl Display for DataControlMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataControlMsg::Alias(alias) => write!(f, "al={alias}"),
        }
    }
}

impl FromStr for AnyControlMsg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ControlMsg::from_str(s)
            .map(AnyControlMsg::Control)
            .or_else(|_| DataControlMsg::from_str(s).map(AnyControlMsg::Data))
    }
}

impl Display for AnyControlMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnyControlMsg::Control(ctrl) => write!(f, "{ctrl}"),
            AnyControlMsg::Data(data) => write!(f, "{data}"),
        }
    }
}

impl From<AnyControlMsg> for Msg {
    fn from(value: AnyControlMsg) -> Self {
        match value {
            AnyControlMsg::Control(ctrl) => Self::Control(ctrl),
            AnyControlMsg::Data(data) => Self::DataControl(data),
        }
    }
}

impl FromArgStr for DataControlMsg {}
impl FromArgStr for AnyControlMsg {}
