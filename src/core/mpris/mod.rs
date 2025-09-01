use std::{path::PathBuf, time::Duration};

use mpris_server::{
    LoopStatus, Metadata, PlaybackRate, PlaybackStatus, PlayerInterface,
    RootInterface, Time, TrackId, Volume,
    zbus::{self, fdo},
};
use tokio::task::JoinHandle;

use crate::{
    core::{
        ControlMsg, DataControlMsg, Msg, Result, RtAndle, UampApp,
        config::{self, CacheSize},
        library::img_lookup::lookup_image_path_rt_thread,
        player::Playback,
    },
    ext::uri::{get_file_uri, parse_file_uri},
};

mod app_impl;

pub struct Mpris {
    rt: RtAndle,
}

impl Mpris {
    pub fn new(rt: RtAndle) -> Self {
        Self { rt }
    }

    async fn send_msg(&self, msg: Msg) -> fdo::Result<()> {
        self.send_msgs(vec![msg]).await
    }

    async fn send_msgs(&self, msgs: Vec<Msg>) -> fdo::Result<()> {
        self.rt
            .msgs_result(msgs)
            .await
            .map_err(|e| fdo::Error::Failed(e.log().to_string()))
    }

    async fn send_zmsg(&self, msg: Msg) -> zbus::Result<()> {
        self.rt
            .msg_result(msg)
            .await
            .map_err(|e| zbus::Error::Failure(e.log().to_string()))
    }

    async fn request<T: Send + 'static>(
        &self,
        f: impl FnOnce(&mut UampApp) -> Result<T> + Send + Sync + 'static,
    ) -> fdo::Result<T> {
        self.rt
            .request(|app, _| f(app))
            .await
            .map_err(|e| fdo::Error::Failed(e.log().to_string()))?
            .map_err(|e| fdo::Error::Failed(e.log().to_string()))
    }
}

impl RootInterface for Mpris {
    async fn raise(&self) -> fdo::Result<()> {
        Ok(())
    }

    async fn quit(&self) -> fdo::Result<()> {
        self.send_msg(Msg::Control(ControlMsg::Close)).await
    }

    async fn can_quit(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn set_fullscreen(&self, _fullscreen: bool) -> zbus::Result<()> {
        Err(zbus::Error::Unsupported)
    }

    async fn can_set_fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn can_raise(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn has_track_list(&self) -> fdo::Result<bool> {
        Ok(false) // TODO: implement track list
    }

    async fn identity(&self) -> fdo::Result<String> {
        Ok(config::APP_ID.to_string())
    }

    async fn desktop_entry(&self) -> fdo::Result<String> {
        Ok("/usr/share/applications/bny-uamp.desktop".to_string())
    }

    async fn supported_uri_schemes(&self) -> fdo::Result<Vec<String>> {
        Ok(vec!["file".to_string()])
    }

    async fn supported_mime_types(&self) -> fdo::Result<Vec<String>> {
        Ok(vec![
            // mp3
            "audio/mpeg".to_string(),
            "audio/mpa".to_string(),
            "audio/mpa-robust".to_string(),
            // flac
            "audio/flac".to_string(),
            // m4a, mp4
            "audio/mp4".to_string(),
            // wav
            "audio/wav".to_string(),
            "audio/vnd.wav".to_string(),
            "audio/x-wav".to_string(),
        ])
    }
}

impl PlayerInterface for Mpris {
    async fn next(&self) -> fdo::Result<()> {
        self.send_msg(Msg::Control(ControlMsg::NextSong(1))).await
    }

    async fn previous(&self) -> fdo::Result<()> {
        self.send_msg(Msg::Control(ControlMsg::PrevSong(None)))
            .await
    }

    async fn pause(&self) -> fdo::Result<()> {
        self.send_msg(Msg::Control(ControlMsg::PlayPause(Some(false))))
            .await
    }

    async fn play_pause(&self) -> fdo::Result<()> {
        self.send_msg(Msg::Control(ControlMsg::PlayPause(None)))
            .await
    }

    async fn stop(&self) -> fdo::Result<()> {
        self.send_msgs(vec![
            Msg::Control(ControlMsg::PlayPause(Some(false))),
            Msg::Control(ControlMsg::SeekTo(Duration::ZERO)),
        ])
        .await
    }

    async fn play(&self) -> fdo::Result<()> {
        self.send_msg(Msg::Control(ControlMsg::PlayPause(Some(true))))
            .await
    }

    async fn seek(&self, offset: Time) -> fdo::Result<()> {
        let dur = Some(Duration::from_millis(offset.abs().as_millis() as u64));
        let msg = if offset.is_positive() {
            ControlMsg::FastForward(dur)
        } else {
            ControlMsg::Rewind(dur)
        };
        self.send_msg(Msg::Control(msg)).await
    }

    async fn set_position(
        &self,
        track_id: TrackId,
        position: Time,
    ) -> fdo::Result<()> {
        if position.is_negative() {
            return Err(fdo::Error::InvalidArgs(
                "Negative song position is not allowed.".to_string(),
            ));
        }

        let Some(idx) = parse_track_id(&track_id) else {
            return Err(fdo::Error::InvalidArgs(
                "Invalid track id.".to_string(),
            ));
        };

        let pos = Duration::from_millis(position.as_millis() as u64);

        let (Some(cur), len) = self
            .request(|app| {
                Ok((
                    app.player.playlist().current_idx(),
                    app.player.timestamp().map(|a| a.total),
                ))
            })
            .await?
        else {
            return Err(fdo::Error::InvalidArgs(
                "Invalid track id.".to_string(),
            ));
        };

        if cur != idx {
            return Err(fdo::Error::InvalidArgs(
                "Invalid track id.".to_string(),
            ));
        }

        let Some(len) = len else {
            return Err(fdo::Error::NotSupported("Cannot seek.".to_string()));
        };

        if pos > len {
            return Err(fdo::Error::InvalidArgs(
                "Cannot seek past track length".to_string(),
            ));
        }

        self.send_msg(Msg::Control(ControlMsg::SeekTo(pos))).await
    }

    async fn open_uri(&self, uri: String) -> fdo::Result<()> {
        let (_, path) = parse_file_uri(&uri).ok_or_else(|| {
            fdo::Error::NotSupported("Unsupported uri.".to_string())
        })?;
        if !path.exists() {
            return Err(fdo::Error::InvalidArgs(format!(
                "The given file `{}` doesn't exist.",
                path.display()
            )));
        }
        let path = path.canonicalize().map_err(|e| {
            fdo::Error::Failed(format!(
                "Failed to canonicalize path `{}`: {e}",
                path.display()
            ))
        })?;
        self.send_msg(DataControlMsg::PlayTmp(vec![path]).into())
            .await
    }

    async fn playback_status(&self) -> fdo::Result<PlaybackStatus> {
        self.request(|app| Ok(playback(app.player.playback_state())))
            .await
    }

    async fn loop_status(&self) -> fdo::Result<LoopStatus> {
        Err(fdo::Error::NotSupported(
            "Uamp doesn't support looping in this way.".to_string(),
        ))
    }

    async fn set_loop_status(
        &self,
        _loop_status: LoopStatus,
    ) -> zbus::Result<()> {
        Err(zbus::Error::Unsupported)
    }

    async fn rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.)
    }

    async fn set_rate(&self, _rate: PlaybackRate) -> zbus::Result<()> {
        Err(zbus::Error::Unsupported)
    }

    async fn shuffle(&self) -> fdo::Result<bool> {
        Err(fdo::Error::NotSupported(
            "Uamp doesn't support shuffle in this way.".to_string(),
        ))
    }

    async fn set_shuffle(&self, _shuffle: bool) -> zbus::Result<()> {
        Err(zbus::Error::Unsupported)
    }

    async fn metadata(&self) -> fdo::Result<Metadata> {
        let (mut meta, task) =
            self.request(|app| Ok(metadata(app, true))).await?;

        if let Some(task) = task
            && let Ok(Ok(img)) = task.await
        {
            meta.set_art_url(Some(get_file_uri("", img)));
        }

        Ok(meta)
    }

    async fn volume(&self) -> fdo::Result<Volume> {
        self.request(|app| Ok(app.player.volume() as f64)).await
    }

    async fn set_volume(&self, volume: Volume) -> zbus::Result<()> {
        self.send_zmsg(Msg::Control(ControlMsg::SetVolume(volume as f32)))
            .await
    }

    async fn position(&self) -> fdo::Result<Time> {
        self.request(|app| Ok(position(app))).await
    }

    async fn minimum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.)
    }

    async fn maximum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.)
    }

    async fn can_go_next(&self) -> fdo::Result<bool> {
        self.request(|app| Ok(app.player.playlist().current().is_some()))
            .await
    }

    async fn can_go_previous(&self) -> fdo::Result<bool> {
        self.request(|app| {
            Ok(!matches!(
                app.player.playlist().current_idx(),
                None | Some(0)
            ))
        })
        .await
    }

    async fn can_play(&self) -> fdo::Result<bool> {
        self.request(|app| Ok(app.player.playlist().current().is_some()))
            .await
    }

    async fn can_pause(&self) -> fdo::Result<bool> {
        self.can_play().await
    }

    async fn can_seek(&self) -> fdo::Result<bool> {
        self.request(
            |app| Ok(app.player.playback_state() != Playback::Stopped),
        )
        .await
    }

    async fn can_control(&self) -> fdo::Result<bool> {
        Ok(true)
    }
}

// This doesn't seem to be supported by KDE so I see no point of properly
// implementing it for my usecase. I also couldn't find anything that would
// actually support this so I won't implement it unless I have a way of testing
// it.
/*impl LocalTrackListInterface for Mpris {
    async fn get_tracks_metadata(
        &self,
        track_ids: Vec<TrackId>,
    ) -> fdo::Result<Vec<Metadata>> {
        self.request(move |app| {
            Ok(track_ids
                .iter()
                .flat_map(|a| parse_track_id(a))
                .map(|i| metadata_for(app, i))
                .collect())
        })
        .await
    }

    async fn add_track(
        &self,
        _uri: Uri,
        _after_track: TrackId,
        _set_as_current: bool,
    ) -> fdo::Result<()> {
        Err(fdo::Error::NotSupported(
            "Uamp doesn't support inserting tracks.".to_string(),
        ))
    }

    async fn remove_track(&self, _track_id: TrackId) -> fdo::Result<()> {
        Err(fdo::Error::NotSupported(
            "Uamp doesn't support removing tracks.".to_string(),
        ))
    }

    async fn go_to(&self, track_id: TrackId) -> fdo::Result<()> {
        let Some(idx) = parse_track_id(&track_id) else {
            return Err(fdo::Error::InvalidArgs(
                "Invalid track id.".to_string(),
            ));
        };
        self.send_msg(ControlMsg::PlaylistJump(idx).into())
    }

    async fn tracks(&self) -> fdo::Result<Vec<TrackId>> {
        let len = self.request(|app| Ok(app.player.playlist().len())).await?;
        Ok((0..len)
            .flat_map(|i| TrackId::try_from(make_track_id(i)))
            .collect())
    }

    async fn can_edit_tracks(&self) -> fdo::Result<bool> {
        Ok(false)
    }
}*/

pub fn playback(pb: Playback) -> PlaybackStatus {
    match pb {
        Playback::Playing => PlaybackStatus::Playing,
        Playback::Paused => PlaybackStatus::Paused,
        Playback::Stopped => PlaybackStatus::Stopped,
    }
}

pub fn metadata(
    app: &UampApp,
    image: bool,
) -> (Metadata, Option<JoinHandle<Result<PathBuf>>>) {
    app.player
        .playlist()
        .current_idx()
        .map(|i| metadata_for(app, i, image))
        .unwrap_or_default()
}

fn metadata_for(
    app: &UampApp,
    idx: usize,
    image: bool,
) -> (Metadata, Option<JoinHandle<Result<PathBuf>>>) {
    let mut data = Metadata::new();
    data.set_trackid(TrackId::try_from(make_track_id(idx)).ok());
    let len = (app.player.playlist().current_idx() == Some(idx))
        .then(|| app.player.timestamp().map(|t| t.total))
        .flatten();
    let Some(id) = app.player.playlist()[..].get(idx) else {
        return (data, None);
    };
    let song = &app.library[id];

    data.set_url(Some(get_file_uri("", song.path())));
    data.set_album(song.album_opt());
    data.set_artist(song.artist_opt().map(|a| [a]));
    data.set_content_created(song.year_str_opt());
    data.set_genre(song.genre_opt().map(|g| [g]));
    data.set_length(len.or(song.length_opt()).map(time_from_dur));
    data.set_title(song.title_opt());
    data.set_disc_number(song.disc_opt().map(|d| d as i32));
    data.set_track_number(song.track_opt().map(|t| t as i32));
    data.set_art_url(
        song.get_cached_path(&app.config, CacheSize::Full)
            .map(|s| get_file_uri("", s)),
    );
    data.set_trackid(
        app.player
            .playlist()
            .current_idx()
            .and_then(|pos| TrackId::try_from(make_track_id(pos)).ok()),
    );

    let task = image.then(|| {
        lookup_image_path_rt_thread(
            app.rt.andle(),
            app.config.cache_path().clone(),
            song.artist().to_string(),
            song.album().to_string(),
            CacheSize::Full,
        )
    });

    (data, task)
}

pub fn position(app: &UampApp) -> Time {
    app.player
        .timestamp()
        .map(|a| a.current)
        .map(time_from_dur)
        .unwrap_or_default()
}

fn time_from_dur(d: Duration) -> Time {
    Time::from_millis(d.as_millis() as i64)
}

fn make_track_id(idx: usize) -> String {
    format!("/uamp/mpris/tid/{idx}")
}

fn parse_track_id(s: &str) -> Option<usize> {
    s.strip_prefix("/uamp/mpris/tid/")
        .and_then(|a| a.parse().ok())
}
