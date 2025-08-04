use raplay::Timestamp;
use serde::Serialize;

use crate::core::{
    player::{AddPolicy, Playback}, server::sub::{PlaylistJump, SetAll, SetPlaylist}, Alias, Result
};

#[derive(Debug, Clone)]
pub enum SubMsg {
    // initial message. Set this values.
    SetAll(SetAll),
    // The current playlist has changed.
    SetPlaylist(SetPlaylist),
    // The playback state has changed.
    Playback(Playback),
    // Jump within the current playlist
    PlaylistJump(PlaylistJump),
    // Seek within the current song
    Seek(Timestamp),
    // Uamp is about to quit
    Quitting,
    // The volume has changed
    SetVolume(f32),
    // The mute state has changed
    SetMute(bool),
    // Playlist has been removed from top of the stack.
    PopPlaylist(PlaylistJump),
    // Combination of PopPlaylist and SetPlaylist
    PopSetPlaylist(SetPlaylist),
    // Sets the playlis add policy of the current playlist
    SetPlaylistAddPolicy(AddPolicy),
    // Sets the playlist end action
    SetPlaylistEndAction(Option<Alias>),
    // Pushes playlist to the playlist stack
    PushPlaylist(SetPlaylist),
}

impl SubMsg {
    pub fn event(&self) -> Result<String> {
        match self {
            Self::SetAll(a) => make_event("set-all", a),
            Self::SetPlaylist(a) => make_event("set-playlist", a),
            Self::Playback(a) => make_event("playback", a),
            Self::PlaylistJump(a) => make_event("playlist-jump", a),
            Self::Seek(a) => make_event("seek", a),
            Self::Quitting => Ok("event: quitting\n\n".to_string()),
            Self::SetVolume(d) => make_event("set-volume", d),
            Self::SetMute(d) => make_event("set-mute", d),
            Self::PopPlaylist(d) => make_event("pop-playlist", d),
            Self::PopSetPlaylist(d) => make_event("pop-set-playlist", d),
            Self::SetPlaylistAddPolicy(d) => {
                make_event("set-playlist-add-policy", d)
            }
            Self::SetPlaylistEndAction(d) => make_event("set-playlist-end-action", d),
            Self::PushPlaylist(d) => make_event("push-playlist", d),
        }
    }
}

fn make_event(n: &str, d: &impl Serialize) -> Result<String> {
    Ok(format!(
        "event: {n}\ndata: {}\n\n",
        serde_json::ser::to_string(d)?
    ))
}
