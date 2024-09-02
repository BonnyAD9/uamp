use raplay::Timestamp;
use serde::{Deserialize, Serialize};

use crate::core::{library::Song, query::Filter};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Request someting from the other side.
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// Request the current playback info.
    Info,
    /// Query for songs
    Query(Filter),
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
    /// The timestamp of the current playback.
    pub timestamp: Option<Timestamp>,
}
