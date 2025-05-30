use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use super::{Msg, Result, UampApp, library::SongId, player::Playlist};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Message to play some specific songs.
#[derive(Clone, Debug)]
pub enum PlayMsg {
    /// Play the given playlist from the given index.
    Playlist(usize, Arc<Vec<SongId>>),
    /// Play the song at the given path.
    TmpPath(Arc<Path>),
}

impl UampApp {
    /// Handle play events.
    pub(in crate::core) fn play_event(
        &mut self,
        msg: PlayMsg,
    ) -> Result<Vec<Msg>> {
        match msg {
            PlayMsg::Playlist(index, songs) => {
                self.player.play_playlist(
                    &mut self.library,
                    Playlist::new(songs, index),
                    true,
                );
            }
            PlayMsg::TmpPath(path) => {
                let id =
                    self.library.add_tmp_path(path.as_ref()).map_err(|e| {
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
            }
        }

        Ok(vec![])
    }
}

impl Serialize for PlayMsg {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        PlayMsgSe::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PlayMsg {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        PlayMsgDe::deserialize(deserializer).map(|m| m.into())
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

#[derive(Serialize)]
#[serde(untagged)]
enum PlayMsgSe<'a> {
    Playlist(usize, &'a Vec<SongId>),
    TmpPath(&'a Path),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PlayMsgDe {
    Playlist(usize, Vec<SongId>),
    TmpPath(PathBuf),
}

impl<'a> From<&'a PlayMsg> for PlayMsgSe<'a> {
    fn from(value: &'a PlayMsg) -> Self {
        match value {
            PlayMsg::Playlist(idx, songs) => Self::Playlist(*idx, songs),
            PlayMsg::TmpPath(path) => Self::TmpPath(path.as_ref()),
        }
    }
}

impl From<PlayMsgDe> for PlayMsg {
    fn from(value: PlayMsgDe) -> Self {
        match value {
            PlayMsgDe::Playlist(idx, songs) => {
                Self::Playlist(idx, Arc::new(songs))
            }
            PlayMsgDe::TmpPath(path) => Self::TmpPath(path.into()),
        }
    }
}
