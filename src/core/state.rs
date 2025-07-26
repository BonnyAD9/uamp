use crate::core::{library::SongId, player::Playback};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    pub playback: Playback,
    pub cur_song: Option<(SongId, usize)>,
    pub volume: f32,
    pub seeked: bool,
}
