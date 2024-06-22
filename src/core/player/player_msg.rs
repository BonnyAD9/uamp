use std::time::Instant;

use crate::core::{library::LibraryUpdate, Msg, UampApp};

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

impl UampApp {
    /// Handles player event messages
    pub fn player_event(&mut self, msg: PlayerMsg) -> Option<Msg> {
        match msg {
            PlayerMsg::SongEnd => {
                self.player.play_next(&mut self.library, 1);
            }
            PlayerMsg::HardPauseAt(i) => self.hard_pause_at = Some(i),
        }
        None
    }

    /// Updates the stored song metadata based on the update level.
    pub fn player_update(&mut self, now: Instant, up: LibraryUpdate) {
        // TODO move logic here
        if up >= LibraryUpdate::RemoveData {
            self.player.remove_deleted(&self.library);
        }

        if let Some(t) = self.hard_pause_at {
            if t <= now {
                self.player.hard_pause();
                self.hard_pause_at = None;
            }
        }
    }
}
