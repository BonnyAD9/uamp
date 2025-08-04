use std::{
    env,
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use itertools::Itertools;
use pareg::{
    ArgErrCtx, ArgError, FromArgStr, has_any_key, mval_arg, split_arg,
    starts_any, val_arg,
};
use serde::{Deserialize, Serialize};

use crate::core::{
    Alias, AppCtrl, Msg, Result, UampApp,
    query::Query,
    server::{
        SubMsg,
        sub::{PlaylistJump, ReorderPlaylistStack},
    },
};

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
    /// Restart uamp with the given binary when that binary becomes available.
    Restart(Option<PathBuf>),
    /// Reorder the playlist stack.
    ReorderPlaylistStack(Vec<usize>),
    /// Play song at the given path as temporary.
    PlayTmp(PathBuf),
}

impl UampApp {
    /// Handles events for [`DataControlMsg`]
    pub(in crate::core) fn data_control_event(
        &mut self,
        _ctrl: &mut AppCtrl,
        msg: DataControlMsg,
    ) -> Result<Vec<Msg>> {
        match msg {
            DataControlMsg::Alias(name) => return self.invoke_alias(&name),
            DataControlMsg::SetPlaylistEndAction(act) => {
                self.player.playlist_mut().on_end = act.clone();
                self.client_update(SubMsg::SetPlaylistEndAction(act));
            }
            DataControlMsg::SetPlaylist(q) => {
                let songs = q.get_ids(
                    &self.library,
                    self.config.simple_sorting(),
                    &self.player,
                )?;
                self.player.play_playlist(
                    &mut self.library,
                    songs.into(),
                    false,
                );
                self.client_update_set_playlist(SubMsg::SetPlaylist);
            }
            DataControlMsg::PushPlaylist(q) => {
                let songs = q.get_ids(
                    &self.library,
                    self.config.simple_sorting(),
                    &self.player,
                )?;
                self.player.push_playlist(
                    &mut self.library,
                    songs.into(),
                    false,
                );
                self.client_update_set_playlist(SubMsg::PushPlaylist);
            }
            DataControlMsg::PushPlaylistAndCur(q) => {
                let songs = q.get_ids(
                    &self.library,
                    self.config.simple_sorting(),
                    &self.player,
                )?;
                self.player.push_with_cur(songs.into());
                self.client_update_set_playlist(SubMsg::PushPlaylistWithCur);
            }
            DataControlMsg::Queue(q) => {
                let songs = q.get_ids(
                    &self.library,
                    self.config.simple_sorting(),
                    &self.player,
                )?;
                self.player.playlist_mut().extend(songs.iter().copied());
                self.client_update(SubMsg::Queue(songs.into()));
            }
            DataControlMsg::PlayNext(q) => {
                let songs = q.get_ids(
                    &self.library,
                    self.config.simple_sorting(),
                    &self.player,
                )?;
                self.player.playlist_mut().play_next(songs.iter().copied());
                self.client_update(SubMsg::PlayNext(songs.into()));
            }
            DataControlMsg::Restart(exe) => {
                self.restart_path = None;
                let exe = if let Some(exe) = exe {
                    exe
                } else {
                    env::current_exe()?
                };
                self.restart_path = Some(exe);
                self.client_update(SubMsg::Restarting);
            }
            DataControlMsg::ReorderPlaylistStack(ord) => {
                self.player.reorder_playlist(&mut self.library, &ord)?;
                self.client_update(SubMsg::ReorderPlaylistStack(
                    ReorderPlaylistStack::new(
                        ord.into(),
                        PlaylistJump::new(&self.player),
                    ),
                ));
            }
            DataControlMsg::PlayTmp(path) => {
                let id = self.library.add_tmp_path(&path).map_err(|e| {
                    e.prepend(format!(
                        "Failed to load song `{:?}`.",
                        path.to_string_lossy()
                    ))
                })?;

                self.player.push_playlist(
                    &mut self.library,
                    vec![id].into(),
                    true,
                );
                self.client_update_tmp_song(id);
            }
        }

        Ok(vec![])
    }
}

impl FromStr for DataControlMsg {
    type Err = ArgError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            v if has_any_key!(v, '=', "al", "alias") => {
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
            v if has_any_key!(v, '=', "restart") => {
                Ok(DataControlMsg::Restart(mval_arg(v, '=')?))
            }
            v if starts_any!(v, "rps=", "reorder-playlist-stack=") => {
                let (_, v) = v.split_once("=").unwrap();
                Ok(DataControlMsg::ReorderPlaylistStack(split_arg(v, ",")?))
            }
            v if has_any_key!(v, '=', "p", "play") => {
                Ok(DataControlMsg::PlayTmp(
                    val_arg::<&Path>(v, '=')?.canonicalize()?,
                ))
            }
            v => ArgError::UnknownArgument(Box::new(ArgErrCtx::from_msg(
                "Unknown control msg.",
                v.to_string(),
            )))
            .err(),
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
            DataControlMsg::Restart(None) => write!(f, "restart"),
            DataControlMsg::Restart(Some(ft)) => {
                write!(f, "restart={}", ft.to_string_lossy())
            }
            DataControlMsg::ReorderPlaylistStack(ord) => {
                write!(f, "{}", ord.iter().map(|a| a.to_string()).join(","))
            }
            DataControlMsg::PlayTmp(p) => write!(f, "p={}", p.display()),
        }
    }
}

impl FromArgStr for DataControlMsg {}
