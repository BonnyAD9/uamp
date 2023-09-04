/// State of the player playback
#[derive(Clone, Copy, Default)]
pub enum Playback {
    /// No song is playing
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
        if play {
            Self::Playing
        } else {
            Self::Paused
        }
    }

    /// Returns true if this is [`Playback::Stopped`]
    pub fn is_stopped(&self) -> bool {
        matches!(self, Playback::Stopped)
    }
}
