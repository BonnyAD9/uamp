use std::sync::Arc;

use raplay::Timestamp;
use serde::Serialize;

use crate::core::{
    Alias, Result,
    player::{AddPolicy, Playback},
    server::sub::{
        Config, InsertIntoPlaylist, NewServer, PlayTmp, PlaylistJump,
        PopPlaylist, PopSetPlaylist, RemoveFromPlaylist, ReorderPlaylistStack,
        SetAll, SetPlaylist,
    },
};

#[derive(Debug, Clone)]
pub enum SubMsg {
    // initial message. Set this values.
    SetAll(Arc<SetAll>),
    // The current playlist has changed.
    SetPlaylist(Arc<SetPlaylist>),
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
    PopPlaylist(Arc<PopPlaylist>),
    // Combination of PopPlaylist and SetPlaylist
    PopSetPlaylist(Arc<PopSetPlaylist>),
    // Sets the playlis add policy of the current playlist
    SetPlaylistAddPolicy(AddPolicy),
    // Sets the playlist end action
    SetPlaylistEndAction(Arc<Option<Alias>>),
    // Pushes playlist to the playlist stack
    PushPlaylist(Arc<SetPlaylist>),
    // Moves the current song to the start of the new playlist. The new
    // playlist in this message aleady contains the moved song.
    PushPlaylistWithCur(Arc<SetPlaylist>),
    // Insert songs into a playlist.
    InsertIntoPlaylist(InsertIntoPlaylist),
    // Removes songs from playlist.
    RemoveFromPlaylist(RemoveFromPlaylist),
    // Uamp is about to restart
    Restarting,
    // Reorders playlist stack. First item in the vector is the top of the
    // queue. Index 0 is the top of the queue. Indexes not present will stay in
    // their relative order at the end of the stack.
    ReorderPlaylistStack(ReorderPlaylistStack),
    // Play temporary song. The new song is added with temporary id as new
    // playlist on top of the stack.
    PlayTmp(Arc<PlayTmp>),
    // New uamp server with different address/port will start. You should
    // reconnect to it. It may not be available immidietely.
    NewServer(NewServer),
    // The path to the client has changed. Client should reload itself.
    ClientChanged,
    // The uamp configuration has changed
    ConfigChanged(Arc<Config>),
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
            Self::SetPlaylistEndAction(d) => {
                make_event("set-playlist-end-action", d)
            }
            Self::PushPlaylist(d) => make_event("push-playlist", d),
            Self::PushPlaylistWithCur(d) => {
                make_event("push-playlist-with-cur", d)
            }
            Self::InsertIntoPlaylist(d) => {
                make_event("insert-into-playlist", d)
            }
            Self::RemoveFromPlaylist(d) => {
                make_event("remove-from-playlist", d)
            }
            Self::Restarting => Ok("event: restarting\n\n".to_string()),
            Self::ReorderPlaylistStack(d) => {
                make_event("reorder-playlist-stack", d)
            }
            Self::PlayTmp(d) => make_event("play-tmp", d),
            Self::NewServer(d) => make_event("new-server", d),
            Self::ClientChanged => Ok("event: client-changed\n\n".to_string()),
            Self::ConfigChanged(d) => make_event("config-changed", d),
        }
    }
}

fn make_event(n: &str, d: &impl Serialize) -> Result<String> {
    Ok(format!(
        "event: {n}\ndata: {}\n\n",
        serde_json::ser::to_string(d)?
    ))
}
