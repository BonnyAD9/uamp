use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    gui::{wid::Command, GuiMessage},
    library::{LibraryMessage, SongId},
    player::PlayerMessage,
};

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
}

/// only simple messages that can be safely send across threads and copied
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
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
    FastForward(Option<f32>),
    /// Seeks backward
    Rewind(Option<f32>),
}

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
        ControlMsg::FastForward(Some(d)) => format!("ff={}", d),
        ControlMsg::Rewind(None) => "rw".to_owned(),
        ControlMsg::Rewind(Some(d)) => format!("rw={}", d),
    }
}

pub enum ComMsg {
    Command(Command),
    Msg(Msg),
}

impl ComMsg {
    pub fn none() -> Self {
        Self::Command(Command::none())
    }

    pub fn tick() -> Self {
        Self::Msg(Msg::Gui(GuiMessage::Tick))
    }
}
