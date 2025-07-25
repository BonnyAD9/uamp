use std::time::Duration;

use futures::{
    StreamExt,
    channel::mpsc::{self, UnboundedSender},
};
use log::error;
use mpris_server::{
    LocalPlayerInterface, LocalRootInterface, LoopStatus, Metadata,
    PlaybackRate, PlaybackStatus, Time, TrackId, Volume,
    zbus::{self, fdo},
};

use crate::{
    core::{
        ControlMsg, FnDelegate, Msg, Result, UampApp,
        config::{self, CacheSize},
        player::Playback,
    },
    env::AppCtrl,
};

mod app_impl;

pub struct Mpris {
    osend: UnboundedSender<Msg>,
}

impl Mpris {
    pub fn new(osend: UnboundedSender<Msg>) -> Self {
        Self { osend }
    }

    fn send_msg(&self, msg: Msg) -> fdo::Result<()> {
        self.osend
            .unbounded_send(msg)
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    fn send_zmsg(&self, msg: Msg) -> zbus::Result<()> {
        self.osend
            .unbounded_send(msg)
            .map_err(|e| zbus::Error::Failure(e.to_string()))
    }

    async fn request<T: Send + 'static>(
        &self,
        f: impl Fn(&mut UampApp) -> Result<T> + Send + Sync + 'static,
    ) -> fdo::Result<T> {
        let (isend, mut irecv) = mpsc::unbounded();
        let delegate = Msg::delegate::<_, FnDelegate<_>>(
            move |app: &mut UampApp, _: &mut AppCtrl| {
                let data = f(app)?;
                if let Err(e) = isend.unbounded_send(data) {
                    error!("Failed to send back mpris data: {e}");
                }
                Ok(vec![])
            },
        );
        self.send_msg(delegate)?;

        irecv.next().await.ok_or_else(|| {
            fdo::Error::Failed("Failed to retrieve data.".to_string())
        })
    }
}

impl LocalRootInterface for Mpris {
    async fn raise(&self) -> fdo::Result<()> {
        Ok(())
    }

    async fn quit(&self) -> fdo::Result<()> {
        self.send_msg(Msg::Control(ControlMsg::Close))
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
        Err(fdo::Error::NotSupported(
            "uamp has no desktop entry".to_string(),
        ))
    }

    async fn supported_uri_schemes(&self) -> fdo::Result<Vec<String>> {
        Ok(vec![]) // TODO: support at least file
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

impl LocalPlayerInterface for Mpris {
    async fn next(&self) -> fdo::Result<()> {
        self.send_msg(Msg::Control(ControlMsg::NextSong(1)))
    }

    async fn previous(&self) -> fdo::Result<()> {
        self.send_msg(Msg::Control(ControlMsg::PrevSong(None)))
    }

    async fn pause(&self) -> fdo::Result<()> {
        self.send_msg(Msg::Control(ControlMsg::PlayPause(Some(false))))
    }

    async fn play_pause(&self) -> fdo::Result<()> {
        self.send_msg(Msg::Control(ControlMsg::PlayPause(None)))
    }

    async fn stop(&self) -> fdo::Result<()> {
        self.send_msg(Msg::Control(ControlMsg::PlayPause(Some(false))))?;
        self.send_msg(Msg::Control(ControlMsg::SeekTo(Duration::ZERO)))
    }

    async fn play(&self) -> fdo::Result<()> {
        self.send_msg(Msg::Control(ControlMsg::PlayPause(Some(true))))
    }

    async fn seek(&self, offset: Time) -> fdo::Result<()> {
        let dur = Some(Duration::from_millis(offset.abs().as_millis() as u64));
        let msg = if offset.is_positive() {
            ControlMsg::FastForward(dur)
        } else {
            ControlMsg::Rewind(dur)
        };
        self.send_msg(Msg::Control(msg))
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

        self.send_msg(Msg::Control(ControlMsg::SeekTo(pos)))
    }

    async fn open_uri(&self, _uri: String) -> fdo::Result<()> {
        // TODO: support this
        Err(fdo::Error::NotSupported(
            "uamp doesn't support opening files yet.".to_string(),
        ))
    }

    async fn playback_status(&self) -> fdo::Result<PlaybackStatus> {
        self.request(|app| Ok(playback(app.player.playback_state())))
            .await
    }

    async fn loop_status(&self) -> fdo::Result<LoopStatus> {
        Ok(LoopStatus::None)
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
        Ok(false)
    }

    async fn set_shuffle(&self, _shuffle: bool) -> zbus::Result<()> {
        Err(zbus::Error::Unsupported)
    }

    async fn metadata(&self) -> fdo::Result<Metadata> {
        self.request(|app| Ok(metadata(app))).await
    }

    async fn volume(&self) -> fdo::Result<Volume> {
        self.request(|app| Ok(app.player.volume() as f64)).await
    }

    async fn set_volume(&self, volume: Volume) -> zbus::Result<()> {
        self.send_zmsg(Msg::Control(ControlMsg::SetVolume(volume as f32)))
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

pub fn playback(pb: Playback) -> PlaybackStatus {
    match pb {
        Playback::Playing => PlaybackStatus::Playing,
        Playback::Paused => PlaybackStatus::Paused,
        Playback::Stopped => PlaybackStatus::Stopped,
    }
}

pub fn metadata(app: &UampApp) -> Metadata {
    let len = app.player.timestamp().map(|t| t.total);
    let Some(id) = app.player.playlist().current() else {
        return Metadata::new();
    };
    let song = &app.library[id];

    let mut data = Metadata::builder()
        .url("file://".to_string() + &song.path().to_string_lossy())
        .build();

    data.set_album(song.album_opt());
    data.set_artist(song.artist_opt().map(|a| [a]));
    data.set_content_created(song.year_str_opt());
    data.set_genre(song.genre_opt().map(|g| [g]));
    data.set_length(len.or(song.length_opt()).map(time_from_dur));
    data.set_title(song.title_opt());
    data.set_disc_number(song.disc_opt().map(|d| d as i32));
    data.set_track_number(song.track_opt().map(|t| t as i32));
    data.set_art_url(
        song.get_cached_path(&app.config, CacheSize::S256)
            .map(|s| "file://".to_string() + &s.to_string_lossy()),
    );
    data.set_trackid(
        app.player
            .playlist()
            .current_idx()
            .and_then(|pos| TrackId::try_from(make_track_id(pos)).ok()),
    );

    data
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
