use std::time::Instant;

use crate::core::{
    library::{Library, LibraryUpdate, SongId},
    Msg, UampApp,
};

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
    pub(in crate::core) fn player_event(
        &mut self,
        msg: PlayerMsg,
    ) -> Vec<Msg> {
        match msg {
            PlayerMsg::SongEnd => {
                self.player.play_next(&mut self.library, 1);
            }
            PlayerMsg::HardPauseAt(i) => self.hard_pause_at = Some(i),
        }
        vec![]
    }

    /// Updates the stored song metadata based on the update level.
    pub(in crate::core) fn player_routine(
        &mut self,
        now: Instant,
        up: LibraryUpdate,
    ) {
        // ReplaceData is handled separately in `player_id_replace`
        if up >= LibraryUpdate::RemoveData && up != LibraryUpdate::ReplaceData
        {
            self.player.retain(|s| !self.library[s].is_deleted());
        }

        if let Some(t) = self.hard_pause_at {
            if t <= now {
                self.player.hard_pause();
                self.hard_pause_at = None;
            }
        }
    }

    /// Old song ids were replaced with new valid song ids.
    pub(in crate::core) fn player_id_replace(
        &mut self,
        n: impl Fn(SongId, &Library) -> bool,
    ) {
        self.player.retain(|s| {
            !self.library[s].is_deleted() && !n(*s, &self.library)
        });
    }
}
