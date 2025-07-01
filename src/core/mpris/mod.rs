mod msg;

use std::{cell::RefCell, time::Duration};

use futures::{
    SinkExt, StreamExt,
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use log::error;
use mpris_server::{
    LocalPlayerInterface, LocalRootInterface, LoopStatus, Metadata,
    PlaybackRate, PlaybackStatus, Time, TrackId, Volume,
    zbus::{self, fdo},
};

use crate::{
    core::{ControlMsg, FnDelegate, Msg, UampApp, mpris::msg::MprisMsg},
    env::AppCtrl,
};

pub struct Mpris {
    isend: UnboundedSender<MprisMsg>,
    irecv: RefCell<UnboundedReceiver<MprisMsg>>,
    osend: UnboundedSender<Msg>,
}

impl Mpris {
    pub fn new(osend: UnboundedSender<Msg>) -> Self {
        let (isend, irecv) = mpsc::unbounded();
        Self {
            isend,
            irecv: irecv.into(),
            osend,
        }
    }

    async fn send_msg(&self, msg: Msg) -> fdo::Result<()> {
        self.osend
            .clone()
            .send(msg)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }

    async fn send_zmsg(&self, msg: Msg) -> zbus::Result<()> {
        self.osend
            .clone()
            .send(msg)
            .await
            .map_err(|e| zbus::Error::Failure(e.to_string()))
    }

    #[allow(clippy::await_holding_refcell_ref)]
    async fn recv(&self) -> fdo::Result<MprisMsg> {
        self.irecv.borrow_mut().next().await.ok_or_else(|| {
            fdo::Error::Failed(
                "Uamp failed to send itself the required data.".to_string(),
            )
        })
    }

    async fn request_playback_status(&self) -> fdo::Result<PlaybackStatus> {
        let isend = self.isend.clone();
        let delegate = Msg::delegate::<_, FnDelegate<_>>(
            move |app: &mut UampApp, _: &mut AppCtrl| {
                let data = if app.player.is_playing() {
                    PlaybackStatus::Playing
                } else {
                    PlaybackStatus::Paused
                };
                if let Err(e) = isend
                    .clone()
                    .unbounded_send(MprisMsg::PlaybackStatus(data))
                {
                    error!("Failed to send back playback status: {e}");
                }
                Ok(vec![])
            },
        );
        self.send_msg(delegate).await?;
        let MprisMsg::PlaybackStatus(status) = self.recv().await? else {
            return Err(fdo::Error::Failed(
                "Failed to retrieve data.".to_string(),
            ));
        };
        Ok(status)
    }

    async fn request_metadata(&self) -> fdo::Result<Metadata> {
        let isend = self.isend.clone();
        let delegate = Msg::delegate::<_, FnDelegate<_>>(
            move |app: &mut UampApp, _: &mut AppCtrl| {
                let Some(id) = app.player.playlist().current() else {
                    if let Err(e) = isend
                        .clone()
                        .unbounded_send(MprisMsg::Metadata(Metadata::new()))
                    {
                        error!("Mpris inner send failed: {e}");
                    }
                    return Ok(vec![]);
                };
                let song = &app.library[id];

                let mut data = Metadata::builder()
                    .album(song.album())
                    // .art_url() TODO
                    .artist([song.artist()])
                    .content_created(song.year_str())
                    .genre([song.genre()])
                    .length(
                        Time::from_millis(song.length().as_millis() as i64),
                    )
                    .title(song.title())
                    .url(
                        "file://".to_string() + &song.path().to_string_lossy(),
                    )
                    .build();

                if song.disc() != u32::MAX {
                    data.set_disc_number(Some(song.disc() as i32));
                }

                if song.track() != u32::MAX {
                    data.set_track_number(Some(song.track() as i32));
                }

                if let Err(e) =
                    isend.clone().unbounded_send(MprisMsg::Metadata(data))
                {
                    error!("Mpris inner send failed: {e}");
                }

                Ok(vec![])
            },
        );
        self.send_msg(delegate).await?;
        let MprisMsg::Metadata(metadata) = self.recv().await? else {
            return Err(fdo::Error::Failed(
                "Failed to retrieve data.".to_string(),
            ));
        };
        Ok(metadata)
    }

    async fn request_volume(&self) -> fdo::Result<Volume> {
        let isend = self.isend.clone();
        let delegate = Msg::delegate::<_, FnDelegate<_>>(
            move |app: &mut UampApp, _: &mut AppCtrl| {
                if let Err(e) = isend.clone().unbounded_send(MprisMsg::Volume(
                    app.player.volume() as f64,
                )) {
                    error!("Failed to send back playback status: {e}");
                }
                Ok(vec![])
            },
        );
        self.send_msg(delegate).await?;
        let MprisMsg::Volume(data) = self.recv().await? else {
            return Err(fdo::Error::Failed(
                "Failed to retrieve data.".to_string(),
            ));
        };
        Ok(data)
    }

    async fn request_position(&self) -> fdo::Result<Time> {
        let isend = self.isend.clone();
        let delegate = Msg::delegate::<_, FnDelegate<_>>(
            move |app: &mut UampApp, _: &mut AppCtrl| {
                let pos = app
                    .player
                    .timestamp()
                    .map(|a| a.current)
                    .unwrap_or_default();
                if let Err(e) =
                    isend.clone().unbounded_send(MprisMsg::Position(
                        Time::from_millis(pos.as_millis() as i64),
                    ))
                {
                    error!("Failed to send back playback status: {e}");
                }
                Ok(vec![])
            },
        );
        self.send_msg(delegate).await?;
        let MprisMsg::Position(data) = self.recv().await? else {
            return Err(fdo::Error::Failed(
                "Failed to retrieve data.".to_string(),
            ));
        };
        Ok(data)
    }
}

impl LocalRootInterface for Mpris {
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
        Ok("uamp".to_string())
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
        self.send_msg(Msg::Control(ControlMsg::PlayPause(Some(false))))
            .await?;
        self.send_msg(Msg::Control(ControlMsg::SeekTo(Duration::ZERO)))
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
        _track_id: TrackId,
        _position: Time,
    ) -> fdo::Result<()> {
        // TODO: support this
        Err(fdo::Error::NotSupported(
            "uamp cannot manage track position yet.".to_string(),
        ))
    }

    async fn open_uri(&self, _uri: String) -> fdo::Result<()> {
        // TODO: support this
        Err(fdo::Error::NotSupported(
            "uamp doesn't support opening files yet.".to_string(),
        ))
    }

    async fn playback_status(&self) -> fdo::Result<PlaybackStatus> {
        self.request_playback_status().await
    }

    async fn loop_status(&self) -> fdo::Result<LoopStatus> {
        Ok(LoopStatus::Playlist)
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
        self.request_metadata().await
    }

    async fn volume(&self) -> fdo::Result<Volume> {
        self.request_volume().await
    }

    async fn set_volume(&self, volume: Volume) -> zbus::Result<()> {
        self.send_zmsg(Msg::Control(ControlMsg::SetVolume(volume as f32)))
            .await
    }

    async fn position(&self) -> fdo::Result<Time> {
        self.request_position().await
    }

    async fn minimum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.)
    }

    async fn maximum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.)
    }

    async fn can_go_next(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_go_previous(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_play(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_pause(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_seek(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_control(&self) -> fdo::Result<bool> {
        Ok(true)
    }
}
