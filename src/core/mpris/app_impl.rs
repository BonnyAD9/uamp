use std::mem;

use mpris_server::{Property, Signal};

use crate::{
    core::{State, UampApp, log_err, mpris},
    env::{AppCtrl, MsgGen},
};

impl UampApp {
    pub fn mpris_routine(&mut self, ctrl: &mut AppCtrl) {
        let Some(mpris) = self.mpris.as_ref().map(|(a, _)| a.clone()) else {
            return;
        };

        let mut old = self.get_state();
        old = mem::replace(&mut self.state, old);

        let changes = self.handle_changes(&mut old);

        if changes.is_empty() && !old.seeked {
            return;
        }

        let seek = old.seeked.then(|| mpris::position(self));

        ctrl.add_stream(MsgGen::new(
            (mpris, changes),
            move |(mpris, changes)| async move {
                if !changes.is_empty() {
                    let change = mpris.properties_changed(changes).await;
                    log_err("Failed to send mpris update: ", change);
                }
                if let Some(position) = seek {
                    let seek = mpris.emit(Signal::Seeked { position }).await;
                    log_err("Failed to send mpris signal: ", seek);
                }
                (None, None)
            },
        ));
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
