use std::{io::ErrorKind, rc::Rc};

use futures::executor::block_on;
use log::info;
use mpris_server::{LocalServer, Property, Signal, zbus};
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::core::{
    AppCtrl, Job, JobMsg, State, UampApp, config, log_err,
    mpris::{self, Mpris},
};

impl UampApp {
    pub fn start_mpris(&mut self, ctrl: &mut AppCtrl) {
        if self.jobs.is_running(Job::SYSTEM_PLAYER) {
            return;
        }

        let e = LocalServer::new(config::APP_ID, Mpris::new(self.rt.clone()));
        let mpris: Option<Rc<_>> =
            log_err("Failed to start mpris player: ", block_on(e))
                .map(|a| a.into());

        self.jobs.system_player = if let Some(s) = mpris {
            let cancel = CancellationToken::new();
            let task = s.run();
            let token = cancel.clone();
            ctrl.task(async move {
                select!(
                    _ = task => unreachable!(),
                    _ = cancel.cancelled() => JobMsg::SystemPlayer.into(),
                )
            });
            self.jobs.run(Job::SYSTEM_PLAYER);
            Some((s, token))
        } else {
            None
        }
    }

    pub fn stop_mpris(&mut self) {
        if let Some((_, cancel)) = self.jobs.system_player.take() {
            cancel.cancel();
        }
    }

    pub fn mpris_routine(&mut self, ctrl: &mut AppCtrl, mut old: State) {
        let Some((mpris, cancel)) = self.jobs.system_player.clone() else {
            return;
        };

        let changes = self.handle_changes(&mut old);

        if changes.is_empty() && !old.seeked {
            return;
        }

        let seek = old.seeked.then(|| mpris::position(self));

        ctrl.spawn(async move {
            let mut restart = false;
            if !changes.is_empty() {
                let change = mpris.properties_changed(changes).await;
                restart |= should_restart(&change);
                log_err("Failed to send mpris update: ", change);
            }
            if let Some(position) = seek {
                let seek = mpris.emit(Signal::Seeked { position }).await;
                restart |= should_restart(&seek);
                log_err("Failed to send mpris signal: ", seek);
            }

            if restart {
                info!("Restarting MPRIS server.");
                cancel.cancel();
            }
        });
    }

    fn handle_changes(&self, old: &mut State) -> Vec<Property> {
        use crate::core::mpris;
        use mpris_server::Property;

        let playback = (old.playback != self.state.playback)
            .then_some(self.state.playback);
        let song = (old.cur_song != self.state.cur_song)
            .then_some(self.state.cur_song.map(|(i, _)| i));
        let volume =
            (old.volume != self.state.volume).then_some(self.state.volume);
        let can_go_next = (old.cur_song.is_some()
            != self.state.cur_song.is_some())
        .then_some(self.state.cur_song.is_some());

        let can_go_previous_old =
            !matches!(old.cur_song.map(|(_, i)| i), None | Some(0));
        let can_go_previous_new =
            !matches!(self.state.cur_song.map(|(_, i)| i), None | Some(0));
        let can_go_previous = (can_go_previous_old != can_go_previous_new)
            .then_some(can_go_previous_new);

        // can_play == can_go_next
        // can_pause == can_play

        let can_seek = (playback.is_some()
            && (old.playback.is_stopped()
                || self.state.playback.is_stopped()))
        .then_some(!self.state.playback.is_stopped());

        old.seeked |= song.is_some();

        let mut properties = vec![];

        properties.extend(
            playback.map(|pb| Property::PlaybackStatus(mpris::playback(pb))),
        );
        properties.extend(
            song.is_some()
                .then(|| Property::Metadata(mpris::metadata(self))),
        );
        properties.extend(volume.map(|v| Property::Volume(v as f64)));
        properties.extend(can_go_next.map(Property::CanGoNext));

        if let Some(c) = can_go_previous {
            properties.extend([
                Property::CanGoPrevious(c),
                Property::CanPause(c),
                Property::CanPause(c),
            ]);
        }

        properties.extend(can_seek.map(Property::CanSeek));

        properties
    }
}

fn should_restart(err: &zbus::Result<()>) -> bool {
    matches!(err, Err(zbus::Error::InputOutput(e)) if e.kind() == ErrorKind::BrokenPipe)
}
