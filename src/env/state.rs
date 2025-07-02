use crate::core::{library::SongId, player::Playback};

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub playback: Playback,
    pub cur_song: Option<(SongId, usize)>,
    pub volume: f32,
}
