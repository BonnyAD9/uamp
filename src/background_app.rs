use futures::{channel::mpsc, StreamExt};
use log::{error, trace};

use crate::{
    core::{config::Config, AnyControlMsg, Msg, Result, UampApp},
    env::{AppCtrl, Command, MsgGen, Streams, UniqueTasks},
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

    let mut streams = Streams::new();
    streams.add(Box::new(MsgGen::new(reciever, |mut r| async {
        let msg = r.next().await.unwrap();
        (Some(r), msg)
    })));

    let mut tasks = UniqueTasks::new(sender.clone());

    for m in init {
        if let Err(e) = sender.unbounded_send(m.into()) {
            error!("Failed to send init message: {e}");
        }
    }

    let mut app =
        UampApp::new(conf, &mut AppCtrl::new(&mut cmd_queue, &tasks), sender)?;

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

        let msg = streams.wait_one();

        for res in tasks.check() {
            trace!("{res:?}");
            #[cfg(debug_assertions)]
            dbg!(&res);
            app.task_end(&mut AppCtrl::new(&mut cmd_queue, &tasks), res);
        }

        app.update(&mut AppCtrl::new(&mut cmd_queue, &tasks), msg);
    }
}
