use std::time::Instant;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Messages sent by the player
#[derive(Clone, Copy, Debug)]
pub enum PlayerMsg {
    /// The song has ended. You can play the next in the playlist. Internal
    /// message DO NOT SEND outside of player.
    SongEnd,
    /// The smooth pause will end at the given moment. You can hard pause when
    /// it passes. Internal message DO NOT SEND outside of player.
    HardPauseAt(Instant),
}
