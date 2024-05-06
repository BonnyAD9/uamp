use log::{error, trace};

use crate::{
    app::UampApp,
    config::Config,
    core::{command::Command, Result},
    tasks::Tasks,
};

pub fn run_background_app(mut conf: Config) -> Result<()> {
    conf.force_server = true;
    let mut app = UampApp::new(conf)?;
    let mut tasks = Tasks::new();
    tasks.add(app.reciever_task()?);

    match app.signal_task() {
        Ok(s) => tasks.add(s),
        Err(e) => error!("Failed to create signal handler: {e}"),
    }

    app.run_server()?;
    loop {
        let msg = tasks.wait_one();

        trace!("{msg:?}");
        #[cfg(debug_assertions)]
        println!("{msg:?}");

        match app.update(msg) {
            Command::None => {}
            Command::Exit => break Ok(()),
        }
    }
}
