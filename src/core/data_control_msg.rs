use std::{fmt::Display, str::FromStr};

use log::error;
use pareg::{
    has_any_key, mval_arg, starts_any, val_arg, ArgError, FromArgStr,
};
use serde::{Deserialize, Serialize};

use crate::env::AppCtrl;

use super::{query::Query, Alias, Error, Msg, UampApp};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Messages that can be safely send across threads, but not necesarily esily
/// copied.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DataControlMsg {
    /// Invoke alias.
    Alias(Alias),
    /// Sets the current playlist end action.
    SetPlaylistEndAction(Option<Alias>),
    /// Sets the current playlist
    SetPlaylist(Query),
    /// Pushes new playlist.
    PushPlaylist(Query),
    /// Pushes new playlist and seamlessly moves the currently playing song
    /// from the current playlist to the start of the new playlist.
    PushPlaylistAndCur(Query),
    /// Add songs specified by the filter to the end of the playlist.
    Queue(Query),
    /// Add songs specified by the filter after the current song in playlist.
    PlayNext(Query),
}

impl UampApp {
    /// Handles events for [`DataControlMsg`]
    pub(in crate::core) fn data_control_event(
        &mut self,
        _ctrl: &mut AppCtrl,
        msg: DataControlMsg,
    ) -> Vec<Msg> {
        match msg {
            DataControlMsg::Alias(name) => match self.invoke_alias(&name) {
                Ok(r) => return r,
                Err(e) => error!("Failed to invoke alias: {e}"),
            },
            DataControlMsg::SetPlaylistEndAction(act) => {
                self.player.playlist_mut().on_end = act;
            }
            DataControlMsg::SetPlaylist(q) => {
                let songs = q.get_ids(
                    &self.library,
                    self.config.simple_sorting(),
                    self.library.iter(),
                );
                self.player.play_playlist(
                    &mut self.library,
                    songs.into(),
                    false,
                );
            }
            DataControlMsg::PushPlaylist(q) => {
                let songs = q.get_ids(
                    &self.library,
                    self.config.simple_sorting(),
                    self.library.iter(),
                );
                self.player.push_playlist(
                    &mut self.library,
                    songs.into(),
                    false,
                );
            }
            DataControlMsg::PushPlaylistAndCur(q) => {
                let songs = q.get_ids(
                    &self.library,
                    self.config.simple_sorting(),
                    self.library.iter(),
                );
                self.player.push_with_cur(songs.into());
            }
            DataControlMsg::Queue(q) => {
                let songs = q.get_ids(
                    &self.library,
                    self.config.simple_sorting(),
                    self.library.iter(),
                );
                self.player.playlist_mut().extend(songs);
            }
            DataControlMsg::PlayNext(q) => {
                let songs = q.get_ids(
                    &self.library,
                    self.config.simple_sorting(),
                    self.library.iter(),
                );
                self.player.playlist_mut().play_next(songs);
            }
        }

        vec![]
    }
}

impl FromStr for DataControlMsg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            v if starts_any!(v, "al", "alias") => {
                Ok(DataControlMsg::Alias(val_arg(v, '=')?))
            }
            v if has_any_key!(
                v,
                '=',
                "spea",
                "pl-end",
                "playlist-end",
                "playlist-end-action"
            ) =>
            {
                Ok(DataControlMsg::SetPlaylistEndAction(mval_arg(v, '=')?))
            }
            v if has_any_key!(v, '=', "set-playlist", "sp") => {
                Ok(DataControlMsg::SetPlaylist(
                    mval_arg(v, '=')?.unwrap_or_default(),
                ))
            }
            v if has_any_key!(v, '=', "push-playlist", "push") => {
                Ok(DataControlMsg::PushPlaylist(
                    mval_arg(v, '=')?.unwrap_or_default(),
                ))
            }
            v if has_any_key!(v, '=', "push-with-cur", "push-cur", "pc") => {
                Ok(DataControlMsg::PushPlaylistAndCur(
                    mval_arg(v, '=')?.unwrap_or_default(),
                ))
            }
            v if has_any_key!(v, '=', "queue", "q") => Ok(
                DataControlMsg::Queue(mval_arg(v, '=')?.unwrap_or_default()),
            ),
            v if has_any_key!(v, '=', "play-next", "queue-next", "qn") => {
                Ok(DataControlMsg::PlayNext(
                    mval_arg(v, '=')?.unwrap_or_default(),
                ))
            }
            v => Err(Error::ArgParse(ArgError::UnknownArgument(
                v.to_owned().into(),
            ))),
        }
    }
}

impl Display for DataControlMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataControlMsg::Alias(alias) => write!(f, "al={alias}"),
            DataControlMsg::SetPlaylistEndAction(None) => write!(f, "spea"),
            DataControlMsg::SetPlaylistEndAction(Some(act)) => {
                write!(f, "spea={act}")
            }
            DataControlMsg::SetPlaylist(ft) => write!(f, "sp={ft}"),
            DataControlMsg::PushPlaylist(ft) => write!(f, "push={ft}"),
            DataControlMsg::PushPlaylistAndCur(ft) => write!(f, "pc={ft}"),
            DataControlMsg::Queue(ft) => write!(f, "q={ft}"),
            DataControlMsg::PlayNext(ft) => write!(f, "qn={ft}"),
        }
    }
}

impl FromArgStr for DataControlMsg {}
