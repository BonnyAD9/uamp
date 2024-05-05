use crate::{app::UampApp, config::Config, core::{command::Command, msg::Msg, Result}, tasks::Tasks};

pub fn run_background_app(mut conf: Config) -> Result<()> {
    conf.force_server = true;
    let mut app = UampApp::new(conf)?;
    let mut tasks = Tasks::new();
    tasks.add(app.create_reciever()?);
    tasks.add(app.create_server()?);
    loop {
        let msg = tasks.wait_one();
        println!("{msg:?}");
        match app.update(msg) {
            Command::None => {},
            Command::Exit => break Ok(()),
        }
    }
}
