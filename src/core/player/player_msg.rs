use std::time::Instant;

/// Messages sent by the player
#[derive(Clone, Copy, Debug)]
pub enum PlayerMsg {
    SongEnd,
    HardPauseAt(Instant),
}
