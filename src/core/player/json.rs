use std::{
    fs::{self, File},
    path::Path,
    time::Duration,
};

use futures::channel::mpsc::UnboundedSender;
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::core::{
    config::Config, library::Library, player::default_volume, Msg, Result,
};

use super::{playback::Playback, sink_wrapper::SinkWrapper, Player, Playlist};

impl Player {
    /// Loads the playback state from json based on the config, returns default
    /// [`Player`] on fail
    pub fn from_config(
        lib: &mut Library,
        sender: UnboundedSender<Msg>,
        conf: &Config,
    ) -> Self {
        if let Some(p) = conf.player_path() {
            Self::from_json(lib, sender, p)
        } else {
            Self::new_default(sender)
        }
    }

    /// Saves the playback state to the default json directory. It doesn't
    /// save the data if it didn't change since the last save.
    ///
    /// # Errors
    /// - cannot create parrent directory
    /// - Failed to serialize
    pub fn to_default_json(&self, closing: bool, conf: &Config) -> Result<()> {
        let save_pos = conf.save_playback_pos().save(closing);
        if !save_pos && !self.get_change() {
            return Ok(());
        }
        if let Some(p) = conf.player_path() {
            self.to_json(save_pos, p)?;
        }
        self.set_change(false);
        Ok(())
    }

    /// Loads the playback state from the given json file, returns default
    /// [`Player`] on fail
    pub fn from_json(
        lib: &mut Library,
        sender: UnboundedSender<Msg>,
        path: impl AsRef<Path>,
    ) -> Self {
        let data = if let Ok(file) = File::open(path.as_ref()) {
            match serde_json::from_reader(file) {
                Ok(p) => p,
                Err(e) => {
                    error!("Failed to load playback info: {e}");
                    PlayerDataLoad::default()
                }
            }
        } else {
            info!("player file {:?} doesn't exist", path.as_ref());
            PlayerDataLoad::default()
        };

        let mut res = Self::new(
            SinkWrapper::new(),
            Playback::Stopped,
            data.playlist,
            data.intercept,
            data.volume,
            data.mute,
            true,
        );

        res.init_inner(sender);
        if let Some(p) = data.position {
            res.play(lib, false);
            res.seek_to(p);
        }
        res
    }
}

/// Used for deserializing the data of the [`Player`]
#[derive(Deserialize)]
struct PlayerDataLoad {
    /// True when the sound is muted, doesn't affect volume
    #[serde(default)]
    mute: bool,
    /// The volume of the playback, doesn't affect mute
    #[serde(default = "default_volume")]
    volume: f32,
    /// The current playlist
    #[serde(default)]
    playlist: Playlist,
    #[serde(default)]
    position: Option<Duration>,
    #[serde(default)]
    intercept: Option<Playlist>,
}

impl Default for PlayerDataLoad {
    fn default() -> Self {
        Self {
            mute: false,
            volume: default_volume(),
            playlist: Playlist::default(),
            position: None,
            intercept: None,
        }
    }
}

/// Used for serializing the data of the [`Player`]
#[derive(Serialize)]
struct PlayerDataSave<'a> {
    /// True when the sound is muted, doesn't affect volume
    mute: bool,
    /// The volume of the playback, doesn't affect mute
    volume: f32,
    /// The current playlist
    playlist: &'a Playlist,
    position: Option<Duration>,
    intercept: Option<&'a Playlist>,
}

impl Player {
    /// Saves the playback state to the given json file
    ///
    /// # Errors
    /// - cannot create parrent directory
    /// - Failed to serialize
    fn to_json(&self, save_pos: bool, path: impl AsRef<Path>) -> Result<()> {
        if let Some(par) = path.as_ref().parent() {
            fs::create_dir_all(par)?;
        }

        let position = save_pos
            .then(|| self.timestamp())
            .flatten()
            .map(|t| t.current);

        serde_json::to_writer(
            File::create(path)?,
            &PlayerDataSave {
                playlist: self.playlist(),
                volume: self.volume(),
                mute: self.mute(),
                position,
                intercept: self.intercept().as_ref(),
            },
        )?;
        Ok(())
    }
}
