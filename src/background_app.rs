use futures::{channel::mpsc, StreamExt};
use log::{error, trace};

use crate::{
    app::UampApp,
    config::Config,
    core::{
        command::{AppCtrl, Command},
        msg::{ControlMsg, Msg},
        Result,
    },
    sync::{
        msg_stream::{MsgGen, Streams},
        tasks::UniqueTasks,
    },
};

pub fn run_background_app(
    mut conf: Config,
    init: Vec<ControlMsg>,
) -> Result<()> {
    conf.force_server = true;
    let mut cmd_queue = vec![];
    let (sender, reciever) = mpsc::unbounded::<Msg>();

    let mut streams = Streams::new();
    let mut tasks = UniqueTasks::new(sender.clone());
    streams.add(Box::new(MsgGen::new(reciever, |mut r| async {
        let msg = r.next().await.unwrap();
        (Some(r), msg)
    })));

    for m in init {
        if let Err(e) = sender.unbounded_send(Msg::Control(m)) {
            error!("Failed to send init message `{m}`: {e}");
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
