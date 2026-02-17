use serde::Deserialize;

use crate::core::{
    Error, Msg, Result, UampApp,
    config::ConfigMsg,
    library::SongId,
    player::Playlist,
    server::{SubMsg, sub::InsertIntoPlaylist},
};

#[derive(Debug, Deserialize)]
pub enum IdControlMsg {
    SetPlaylist {
        songs: Vec<SongId>,
        position: usize,
        play: bool,
    },
    PushPlaylist {
        songs: Vec<SongId>,
        position: usize,
        play: bool,
    },
    InsertIntoPlaylist {
        songs: Vec<SongId>,
        /// Position at which to insert.
        position: usize,
        /// Playlist index in playlist stack.
        playlist: usize,
    },
    SetConfig(serde_json::Value),
}

impl UampApp {
    pub fn id_control_event(
        &mut self,
        event: IdControlMsg,
    ) -> Result<Vec<Msg>> {
        match event {
            IdControlMsg::SetPlaylist {
                mut songs,
                position,
                play,
            } => {
                songs.retain(|s| !self.library[s].is_deleted());
                self.player.play_playlist(
                    &mut self.library,
                    Playlist::new(songs, position),
                    play,
                );
                self.client_update_set_playlist(|a| {
                    SubMsg::SetPlaylist(a.into())
                });
            }
            IdControlMsg::PushPlaylist {
                mut songs,
                position,
                play,
            } => {
                songs.retain(|s| !self.library[s].is_deleted());
                self.player.push_playlist(
                    &mut self.library,
                    Playlist::new(songs, position),
                    play,
                );
                self.client_update_set_playlist(|a| {
                    SubMsg::PushPlaylist(a.into())
                });
            }
            IdControlMsg::InsertIntoPlaylist {
                mut songs,
                position,
                playlist,
            } => {
                let Some(plist) = self.player.get_playlist_mut(playlist)
                else {
                    return Error::invalid_operation()
                        .msg("Invalid playlist index.")
                        .err();
                };
                if position > plist.len() {
                    return Error::invalid_operation()
                        .msg("Invalid position within playlist.")
                        .err();
                }
                songs.retain(|s| !self.library[s].is_deleted());
                plist.insert(position, &songs);
                self.client_update(SubMsg::InsertIntoPlaylist(
                    InsertIntoPlaylist::new(songs, position, playlist),
                ));
            }
            IdControlMsg::SetConfig(cfg) => {
                return Ok(vec![ConfigMsg::Set(cfg).into()]);
            }
        }
        Ok(vec![])
    }
}
