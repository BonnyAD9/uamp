use core::fmt::Debug;
use std::{sync::Arc, time::Duration};

use iced::window;
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{
    app::UampApp,
    config::ConfMessage,
    gui::{wid::Command, GuiMessage, WinMessage},
    library::{LibraryMessage, SongId},
    player::PlayerMessage,
};

use super::{extensions::duration_to_string, Error};

/// Event messages in uamp
#[allow(missing_debug_implementations)]
#[derive(Clone, Debug)]
pub enum Msg {
    /// Play song song at the given index in the playlist
    PlaySong(usize, Arc<[SongId]>),
    /// Some simple messages
    Control(ControlMsg),
    /// Gui messages handled by the gui
    Gui(GuiMessage),
    /// Player messges handled by the player
    Player(PlayerMessage),
    /// Library messages handled by the library
    Library(LibraryMessage),
    /// Dellegate the message
    Delegate(Arc<dyn MessageDelegate>),
    /// The window has changed its parameters
    WindowChange(WinMessage),
    Config(ConfMessage),
    Init,
    /// Do nothing
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
    PrevSong(usize),
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
}

/// Message returned after proccessing message, either starts a iced command,
/// or produces another message
pub enum ComMsg {
    /// Message that produces iced command
    Command(Command),
    /// Message that produces another message
    Msg(Msg),
}

impl ComMsg {
    /// Returns message that doesn't do anything
    pub fn none() -> Self {
        Self::Command(Command::none())
    }

    /// Returns message that ticks the gui tick
    pub fn tick() -> Self {
        Self::Msg(Msg::Gui(GuiMessage::Tick))
    }
}

impl UampApp {
    /// handles the control events
    pub fn control_event(&mut self, msg: ControlMsg) -> ComMsg {
        match msg {
            ControlMsg::PlayPause(p) => {
                self.player.play_pause(
                    &mut self.library,
                    p.unwrap_or(!self.player.is_playing()),
                );
                return ComMsg::tick();
            }
            ControlMsg::NextSong(n) => {
                self.player.play_next(&mut self.library, n);
                return ComMsg::tick();
            }
            ControlMsg::PrevSong(n) => {
                self.player.play_prev(&mut self.library, n);
                if let Err(e) = self.config.delete_old_logs() {
                    error!("Failed to remove logs: {e}");
                }
                return ComMsg::tick();
            }
            ControlMsg::Close => {
                self.save_all();
                if self.library.any_process() {
                    self.pending_close = true;
                    return ComMsg::none();
                }
                return ComMsg::Command(window::close());
            }
            ControlMsg::Shuffle => self.player.shuffle(),
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
                return ComMsg::tick();
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
                if let Some(ts) = self.player.timestamp() {
                    if ts.total.checked_sub(d).unwrap_or_default()
                        < Duration::from_millis(100)
                    {
                        return ComMsg::Msg(Msg::Control(
                            ControlMsg::NextSong(1),
                        ));
                    }
                }
                return ComMsg::tick();
            }
            ControlMsg::FastForward(d) => {
                if let Some(ts) = self.player.timestamp() {
                    let pos =
                        ts.current + d.unwrap_or(self.config.seek_jump().0);
                    let pos = pos.min(ts.total);
                    if ts.total.checked_sub(pos).unwrap_or_default()
                        < Duration::from_millis(100)
                    {
                        return ComMsg::Msg(Msg::Control(
                            ControlMsg::NextSong(1),
                        ));
                    }
                    self.player.seek_to(pos);
                    return ComMsg::tick();
                }
            }
            ControlMsg::Rewind(d) => {
                if let Some(ts) = self.player.timestamp() {
                    let pos = ts
                        .current
                        .checked_sub(d.unwrap_or(self.config.seek_jump().0))
                        .unwrap_or(Duration::ZERO);
                    self.player.seek_to(pos);
                    return ComMsg::tick();
                }
            }
        };

        ComMsg::none()
    }
}

/// The reverse of parsing control message (e.g. from cli)
pub fn get_control_string(m: &ControlMsg) -> String {
    match m {
        ControlMsg::PlayPause(None) => "pp".to_owned(),
        ControlMsg::PlayPause(Some(v)) => {
            if *v { "pp=play" } else { "pp=pause" }.to_owned()
        }
        ControlMsg::NextSong(v) => format!("ns={v}"),
        ControlMsg::PrevSong(v) => format!("ps={v}"),
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
    }
}

pub trait MessageDelegate: Sync + Send + Debug {
    fn update(&self, app: &mut UampApp) -> ComMsg;
}

pub struct FnDelegate<T>(T)
where
    T: Sync + Send + Fn(&mut UampApp) -> ComMsg;

impl<T> MessageDelegate for FnDelegate<T>
where
    T: Sync + Send + Fn(&mut UampApp) -> ComMsg,
{
    fn update(&self, app: &mut UampApp) -> ComMsg {
        self.0(app)
    }
}

impl<T> Debug for FnDelegate<T>
where
    T: Sync + Send + Fn(&mut UampApp) -> ComMsg,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FnDelegate").finish()
    }
}

impl<T> From<T> for FnDelegate<T>
where
    T: Sync + Send + Fn(&mut UampApp) -> ComMsg,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}
