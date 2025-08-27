use serde::Deserialize;

use crate::core::{
    Msg, Result, UampApp,
    config::ConfigMsg,
    library::SongId,
    player::Playlist,
    server::{SubMsg, sub},
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
    SetConfig(Box<sub::Config>),
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
            IdControlMsg::SetConfig(cfg) => {
                return Ok(vec![ConfigMsg::Set(cfg).into()]);
            }
        }
        Ok(vec![])
    }
}
