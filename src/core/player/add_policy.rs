use serde::{Deserialize, Serialize};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Describes how to add songs to a playlist.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AddPolicy {
    /// Add songs to the end of the playlist.
    End,
    /// Add songs after the current playing song.
    Next,
    /// Mix the songs randomly after the currently playing song.
    MixIn,
}
