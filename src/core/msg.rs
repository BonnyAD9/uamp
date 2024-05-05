use core::fmt::Debug;
use std::{
    mem::replace,
    sync::Arc,
    time::{Duration, Instant},
};

use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{
    app::UampApp,
    config::ConfMessage,
    library::{Filter, LibraryMessage, SongId},
    player::PlayerMessage,
};

use super::{
    command::{ComMsg, Command},
    extensions::duration_to_string,
    Error,
};

/// Event messages in uamp
#[allow(missing_debug_implementations)]
#[derive(Clone, Debug)]
pub enum Msg {
    /// Play song song at the given index in the playlist
    _PlaySong(usize, Arc<[SongId]>),
    /// Some simple messages
    Control(ControlMsg),
    /// Player messges handled by the player
    Player(PlayerMessage),
    /// Library messages handled by the library
    Library(LibraryMessage),
    /// Dellegate the message
    Delegate(Arc<dyn MessageDelegate>),
    Config(ConfMessage),
    // General update
    Tick,
    Init,
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
    LoadNewSongs,
    /// Seek to the given timesamp
    SeekTo(Duration),
    /// Seeks forward
    FastForward(Option<Duration>),
    /// Seeks backward
    Rewind(Option<Duration>),
    /// Sets the current playlist
    SetPlaylist(Filter),
    /// Thriggers save
    Save,
}

impl UampApp {
    /// handles the control events
    pub fn control_event(&mut self, msg: ControlMsg) -> ComMsg<Msg> {
        match msg {
            ControlMsg::PlayPause(p) => {
                let pp = p.unwrap_or(!self.player.is_playing());
                if pp {
                    self.hard_pause_at = None;
                }
                self.player.play_pause(&mut self.library, pp);
                return ComMsg::Msg(Msg::Tick);
            }
            ControlMsg::NextSong(n) => {
                self.player.play_next(&mut self.library, n);
                return ComMsg::Msg(Msg::Tick);
            }
            ControlMsg::PrevSong(n) => {
                if let Some(t) = self.config.previous_timeout() {
                    if n.is_none() {
                        let now = Instant::now();
                        if now - replace(&mut self.last_prev, now) >= t.0 {
                            return ComMsg::Msg(Msg::Control(
                                ControlMsg::SeekTo(Duration::ZERO),
                            ));
                        }
                    }
                }

                self.player.play_prev(&mut self.library, n.unwrap_or(1));
                if let Err(e) = self.config.delete_old_logs() {
                    error!("Failed to remove logs: {e}");
                }
                return ComMsg::Msg(Msg::Tick);
            }
            ControlMsg::Close => {
                self.save_all();
                if self.library.any_process() {
                    self.pending_close = true;
                    return ComMsg::none();
                }
                // return ComMsg::Command(window::close());
                return ComMsg::Command(Command::Exit);
            }
            ControlMsg::Shuffle => {
                self.player.shuffle();
                return ComMsg::Msg(Msg::Tick);
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
                self.player.play_at(
                    &mut self.library,
                    i,
                    self.player.is_playing(),
                );
                return ComMsg::Msg(Msg::Tick);
            }
            ControlMsg::Mute(b) => {
                self.player.set_mute(b.unwrap_or(!self.player.mute()))
            }
            ControlMsg::LoadNewSongs => {
                match self
                    .library
                    .start_get_new_songs(&self.config, self.sender.clone())
                {
                    Err(e) if matches!(e, Error::InvalidOperation(_)) => {
                        info!("Cannot load new songs: {e}")
                    }
                    Err(e) => error!("Cannot load new songs: {e}"),
                    _ => {}
                }
            }
            ControlMsg::SeekTo(d) => {
                self.player.seek_to(d);
                return ComMsg::Msg(Msg::Tick);
            }
            ControlMsg::FastForward(d) => {
                let t = d.unwrap_or(self.config.seek_jump().0);
                self.player.seek_by(t, true);
                return ComMsg::Msg(Msg::Tick);
            }
            ControlMsg::Rewind(d) => {
                let t = d.unwrap_or(self.config.seek_jump().0);
                self.player.seek_by(t, false);
                return ComMsg::Msg(Msg::Tick);
            }
            ControlMsg::SetPlaylist(filter) => {
                let songs: Vec<_> = self.library.filter(filter).collect();
                self.player.play_playlist(
                    &mut self.library,
                    songs,
                    None,
                    false,
                );
            }
            ControlMsg::Save => self.save_all(),
        };

        ComMsg::none()
    }
}

/// The reverse of parsing control message (e.g. from cli)
pub fn _get_control_string(m: &ControlMsg) -> String {
    match m {
        ControlMsg::PlayPause(None) => "pp".to_owned(),
        ControlMsg::PlayPause(Some(v)) => {
            if *v { "pp=play" } else { "pp=pause" }.to_owned()
        }
        ControlMsg::NextSong(v) => format!("ns={v}"),
        ControlMsg::PrevSong(None) => "ps".into(),
        ControlMsg::PrevSong(Some(v)) => format!("ps={v}"),
        ControlMsg::SetVolume(v) => format!("v={v}"),
        ControlMsg::VolumeUp(None) => "vu".to_owned(),
        ControlMsg::VolumeUp(Some(v)) => format!("vu={v}"),
        ControlMsg::VolumeDown(None) => "vd".to_owned(),
        ControlMsg::VolumeDown(Some(v)) => format!("vd={v}"),
        ControlMsg::Mute(None) => "mute".to_owned(),
        ControlMsg::Mute(Some(v)) => format!("mute={v}"),
        ControlMsg::Shuffle => "shuffle".to_owned(),
        ControlMsg::PlaylistJump(v) => format!("pj={v}"),
        ControlMsg::Close => "x".to_owned(),
        ControlMsg::LoadNewSongs => "load-songs".to_owned(),
        ControlMsg::SeekTo(d) => format!("st={}", d.as_secs_f32()),
        ControlMsg::FastForward(None) => "ff".to_owned(),
        ControlMsg::FastForward(Some(d)) => {
            format!("ff={}", duration_to_string(*d, false))
        }
        ControlMsg::Rewind(None) => "rw".to_owned(),
        ControlMsg::Rewind(Some(d)) => {
            format!("rw={}", duration_to_string(*d, false))
        }
        ControlMsg::SetPlaylist(_) => "sp".to_owned(),
        ControlMsg::Save => "save".to_owned(),
    }
}

pub trait MessageDelegate: Sync + Send + Debug {
    fn update(&self, app: &mut UampApp) -> ComMsg<Msg>;
}

pub struct FnDelegate<T>(T)
where
    T: Sync + Send + Fn(&mut UampApp) -> ComMsg<Msg>;

impl<T> MessageDelegate for FnDelegate<T>
where
    T: Sync + Send + Fn(&mut UampApp) -> ComMsg<Msg>,
{
    fn update(&self, app: &mut UampApp) -> ComMsg<Msg> {
        self.0(app)
    }
}

impl<T> Debug for FnDelegate<T>
where
    T: Sync + Send + Fn(&mut UampApp) -> ComMsg<Msg>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FnDelegate").finish()
    }
}

impl<T> From<T> for FnDelegate<T>
where
    T: Sync + Send + Fn(&mut UampApp) -> ComMsg<Msg>,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}
