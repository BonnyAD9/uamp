use eyre::Result;
use iced::{Application, Settings};
use uamp_app::UampApp;

mod config;
mod fancy_widgets;
mod library;
mod player;
mod song;
mod theme;
mod uamp_app;

fn main() -> Result<()> {
    UampApp::run(Settings::default())?;
    Ok(())
}
