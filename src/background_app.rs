use log::{error, trace};
use tokio::runtime;

use crate::{
    core::{AnyControlMsg, Error, Msg, Result, UampApp, config::Config},
    env::{AppCtrl, Command, rt},
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Runs uamp as background app that must have server so that it can be
/// controlled.
pub fn run_background_app(
    conf: Config,
    init: Vec<AnyControlMsg>,
) -> Result<()> {
    let rt = runtime::Builder::new_current_thread().enable_io().build()?;
    let local = tokio::task::LocalSet::new();
    rt.block_on(local.run_until(run_bg_async(conf, init)))
}

async fn run_bg_async(
    mut conf: Config,
    init: Vec<AnyControlMsg>,
) -> Result<()> {
    conf.force_server = Some(true);
    let mut cmd_queue = vec![];

    let (mut rt, handle) = rt::make_rt::<Msg, Error>();

    if !init.is_empty() {
        handle.msgs(init.into_iter().map(|a| a.into()).collect());
    }

    let mut app =
        UampApp::new(conf, &mut AppCtrl::new(&mut cmd_queue), handle)?;

    'mainloop: loop {
        for cmd in cmd_queue.drain(..) {
            trace!("{cmd:?}");
            #[cfg(debug_assertions)]
            dbg!(&cmd);
            match cmd {
                Command::Exit => break 'mainloop Ok(()),
                Command::AddStrem(s) => rt.add_stream(s),
            }
        }

        let msg = rt
            .next()
            .await
            .expect("The runtime stopped. This shouldn't happen.");

        match msg {
            rt::Msg::Msg(msgs, rsend) => {
                let res = app
                    .update_many(&mut AppCtrl::new(&mut cmd_queue), msgs)
                    .map_err(|e| e.log());
                if let Err(ref e) = res {
                    error!("{e}");
                }
                if let Some(rsend) = rsend {
                    _ = rsend.send(res);
                }
            }
            rt::Msg::AddStream(s) => rt.add_stream(s),
        }
    }
}
