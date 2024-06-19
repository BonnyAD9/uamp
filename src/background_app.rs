use futures::{channel::mpsc, future, StreamExt};
use log::{error, trace};

use crate::{
    core::{config::Config, AnyControlMsg, Msg, Result, UampApp},
    env::{AppCtrl, Command, MsgGen, Streams, UniqueTasks},
};

pub fn run_background_app(
    mut conf: Config,
    init: Vec<AnyControlMsg>,
) -> Result<()> {
    conf.force_server = true;
    let mut cmd_queue = vec![];
    let (sender, reciever) = mpsc::unbounded::<Msg>();

    let mut streams = Streams::new();
    // I don't know why, but adding this stream that does nothing to the streams
    // reduces usage of main thread from 100% to basicly 0% :)
    streams.add(Box::new(MsgGen::new((), |_| async {
        let msg = future::pending::<Msg>().await;
        (Some(()), msg)
    })));
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
            println!("{cmd:?}");
            match cmd {
                Command::Exit => break 'mainloop Ok(()),
                Command::AddStream(stream) => streams.add(stream),
                Command::AddTask(typ, task) => {
                    if let Err(e) = tasks.add(typ, task) {
                        error!("Failed to start task: {e}");
                    }
                }
            }
        }

        let msg = streams.wait_one();

        trace!("{msg:?}");
        #[cfg(debug_assertions)]
        println!("{msg:?}");

        for res in tasks.check() {
            trace!("{res:?}");
            #[cfg(debug_assertions)]
            println!("{res:?}");
            app.task_end(&mut AppCtrl::new(&mut cmd_queue, &tasks), res);
        }

        app.update(&mut AppCtrl::new(&mut cmd_queue, &tasks), msg);
    }
}
