use raplay::Timestamp;
use serde::{Deserialize, Serialize};

use crate::core::{Alias, library::Song, player::AddPolicy, query::Query};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Request someting from the other side.
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// Request the current playback info.
    Info(usize, usize),
    /// Query for songs
    Query(Query),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DataResponse {
    Info(Box<Info>),
    SongList(Vec<Song>),
}

/// Info about the playback.
#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    /// Uamp version.
    pub version: String,
    /// Song that is now playing.
    pub now_playing: Option<Song>,
    /// Length of the playlisg.
    pub playlist_len: usize,
    /// Current position in the playlist.
    pub playlist_pos: Option<usize>,
    /// True if is currently playing.
    pub is_playing: bool,
    /// The current volume
    pub volume: f32,
    /// If uamp is muted
    pub mute: bool,
    /// The timestamp of the current playback.
    pub timestamp: Option<Timestamp>,
    /// Songs in the playlist before
    pub before: Vec<Song>,
    /// Songs in the playlist after
    pub after: Vec<Song>,
    /// Playlist stack (position, length)
    pub playlist_stack: Vec<(Option<usize>, usize)>,
    /// Playlist end action
    pub playlist_end: Option<Alias>,
    /// Playlist add policy
    pub playlist_add_policy: AddPolicy,
}
