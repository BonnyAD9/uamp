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
    pub fn play(play: bool) -> Self {
        if play {
            Self::Playing
        } else {
            Self::Paused
        }
    }

    pub fn is_stopped(&self) -> bool {
        matches!(self, Playback::Stopped)
    }
}
