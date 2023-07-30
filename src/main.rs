use eyre::Result;
use iced::{window, Application, Settings};
use log::{error, info};
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

    UampApp::run(make_settings())?;
    Ok(())
}

fn make_settings() -> Settings<()> {
    let icon = window::icon::from_rgba(
        include_bytes!("../assets/raw_img/icon_64.data")
            .to_owned()
            .into(),
        64,
        64,
    );

    if let Err(e) = &icon {
        error!("Failed to set the icon: {e}");
    }

    Settings {
        window: window::Settings {
            icon: icon.ok(),
            ..Default::default()
        },
        id: Some("uamp".to_owned()),
        ..Default::default()
    }
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
