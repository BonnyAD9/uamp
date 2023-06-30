use eyre::Result;
use iced::{window, Application, Settings};
use log::info;
use uamp_app::UampApp;

mod config;
mod fancy_widgets;
mod library;
mod player;
mod song;
mod theme;
mod uamp_app;
mod uamp_gui;
mod wid;

fn main() -> Result<()> {
    if let Err(e) = start_logger() {
        eprintln!("failed to start logger: {e}");
    }

    info!("started");

    let mut set = Settings::default();
    set.window.icon = window::icon::from_rgba(
        include_bytes!("../assets/raw_img/icon_64.data")
            .to_owned()
            .into(),
        64,
        64,
    ).ok();

    UampApp::run(set)?;
    Ok(())
}

fn start_logger() -> Result<()> {
    flexi_logger::Logger::try_with_env()?
        .log_to_file(
            flexi_logger::FileSpec::default()
                .directory(config::default_config_path().join("log")),
        )
        .write_mode(flexi_logger::WriteMode::BufferAndFlush)
        .start()?;
    Ok(())
}
