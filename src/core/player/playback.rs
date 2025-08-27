//===========================================================================//
//                                   Public                                  //
//===========================================================================//

use serde::Serialize;

/// State of the player playback
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Serialize)]
pub enum Playback {
    /// No song is playing (no song is loaded)
    #[default]
    Stopped,
    /// Song is playing
    Playing,
    /// Song is paused
    Paused,
}

impl Playback {
    /// Creates [`Playback::Playing`] or [`Playback::Paused`]
    pub fn play(play: bool) -> Self {
        if play { Self::Playing } else { Self::Paused }
    }

    /// Returns true if this is [`Playback::Stopped`]
    pub fn is_stopped(&self) -> bool {
        matches!(self, Playback::Stopped)
    }
}

impl From<bool> for Playback {
    fn from(value: bool) -> Self {
        Self::play(value)
    }
}

impl From<Option<bool>> for Playback {
    fn from(value: Option<bool>) -> Self {
        value.map_or(Playback::Stopped, Playback::play)
    }
}
