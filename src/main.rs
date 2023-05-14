use eyre::Result;
use iced::{Application, Settings};
use uamp_app::UampApp;

mod config;
mod library;
mod player;
mod song;
mod uamp_app;
mod wrap_box;

fn main() -> Result<()> {
    UampApp::run(Settings::default())?;
    Ok(())
}
