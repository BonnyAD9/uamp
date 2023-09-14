use std::time::Duration;

use super::{app::MainPage, settings::SetMessage};

/// A gui message
#[derive(Clone, Debug)]
pub enum Message {
    /// Jump to the main page
    SetPage(MainPage),
    /// Seek slider is dragged by the user
    SeekSliderMove(Duration),
    /// The user stopped dragging the slider
    SeekSliderEnd,
    /// Ticks every set amount of time (used for example for the seek slider)
    Tick,
    Setings(SetMessage),
}

/// Window has changed its parameters
#[derive(Clone, Copy, Debug)]
pub enum WinMessage {
    /// The window has moved
    Position(i32, i32),
    /// The window has resized
    Size(u32, u32),
}
