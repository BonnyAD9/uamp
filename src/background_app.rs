#[cfg(unix)]
use futures::executor::block_on;
use futures::{StreamExt, channel::mpsc};
use log::{error, trace};
#[cfg(unix)]
use mpris_server::Property;

use crate::{
    core::{AnyControlMsg, Msg, Result, UampApp, config::Config},
    env::{AppCtrl, Command, MsgGen, Streams, UniqueTasks},
};
#[cfg(unix)]
use crate::{
    core::{config, mpris::Mpris},
    env::State,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Runs uamp as background app that must have server so that it can be
/// controlled.
pub fn run_background_app(
    mut conf: Config,
    init: Vec<AnyControlMsg>,
) -> Result<()> {
    conf.force_server = true;
    let mut cmd_queue = vec![];
    let (sender, reciever) = mpsc::unbounded::<Msg>();

    #[cfg(unix)]
    let server = block_on(mpris_server::LocalServer::new(
        config::APP_ID,
        Mpris::new(sender.clone()),
    ))?;

    let mut streams = Streams::new();
    streams.add(Box::new(MsgGen::new(reciever, |mut r| async {
        let msg = r.next().await.unwrap();
        (Some(r), msg)
    })));

    #[cfg(unix)]
    streams.add(Box::new(MsgGen::new(server.run(), |s| async {
        s.await;
        unreachable!()
    })));

    let mut tasks = UniqueTasks::new(sender.clone());

    for m in init {
        if let Err(e) = sender.unbounded_send(m.into()) {
            error!("Failed to send init message: {e}");
        }
    }

    let mut seeked = false;

    let mut app = UampApp::new(
        conf,
        &mut AppCtrl::new(&mut cmd_queue, &tasks, &mut seeked),
        sender,
    )?;

    #[cfg(unix)]
    let mut last_state = app.get_state();

    'mainloop: loop {
        for cmd in cmd_queue.drain(..) {
            trace!("{cmd:?}");
            #[cfg(debug_assertions)]
            dbg!(&cmd);
            match cmd {
                Command::Exit => break 'mainloop Ok(()),
                Command::_AddStream(stream) => streams.add(stream),
                Command::AddTask(typ, task) => {
                    if let Err(e) = tasks.add(typ, task) {
                        error!("Failed to start task: {}", e.log());
                    }
                }
            }
        }

        #[cfg(unix)]
        {
            let state = app.get_state();
            let changes =
                handle_changes(&app, &last_state, &state, &mut seeked);
            block_on(server.properties_changed(changes))?;
            if seeked {
                use mpris_server::Signal;

                use crate::core::mpris;

                block_on(server.emit(Signal::Seeked {
                    position: mpris::position(&app),
                }))?;
            }
            seeked = false;
            last_state = state;
        }

        let msg = streams.wait_one();

        for res in tasks.check() {
            trace!("{res:?}");
            #[cfg(debug_assertions)]
            dbg!(&res);
            app.task_end(
                &mut AppCtrl::new(&mut cmd_queue, &tasks, &mut seeked),
                res,
            );
        }

        app.update(
            &mut AppCtrl::new(&mut cmd_queue, &tasks, &mut seeked),
            msg,
        );
    }
}

#[cfg(unix)]
fn handle_changes(
    app: &UampApp,
    org: &State,
    new: &State,
    seeked: &mut bool,
) -> Vec<Property> {
    use crate::core::mpris;
    use mpris_server::Property;

    let playback = (org.playback != new.playback).then_some(new.playback);
    let song =
        (org.cur_song != new.cur_song).then_some(new.cur_song.map(|(i, _)| i));
    let volume = (org.volume != new.volume).then_some(new.volume);
    let can_go_next = (org.cur_song.is_some() != new.cur_song.is_some())
        .then_some(new.cur_song.is_some());

    let can_go_previous_old =
        !matches!(org.cur_song.map(|(_, i)| i), None | Some(0));
    let can_go_previous_new =
        !matches!(new.cur_song.map(|(_, i)| i), None | Some(0));
    let can_go_previous = (can_go_previous_old != can_go_previous_new)
        .then_some(can_go_previous_new);

    // can_play == can_go_next
    // can_pause == can_play

    let can_seek = (playback.is_some()
        && (org.playback.is_stopped() || new.playback.is_stopped()))
    .then_some(!new.playback.is_stopped());

    *seeked |= song.is_some();

    let mut properties = vec![];

    properties.extend(
        playback.map(|pb| Property::PlaybackStatus(mpris::playback(pb))),
    );
    properties.extend(
        song.is_some()
            .then(|| Property::Metadata(mpris::metadata(app))),
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
