use std::time::Duration;

use super::app::MainPage;

/// A gui message
#[derive(Clone, Copy, Debug)]
pub enum Message {
    /// Jump to the main page
    SetPage(MainPage),
    /// Seek slider is dragged by the user
    SeekSliderMove(Duration),
    /// The user stopped dragging the slider
    SeekSliderEnd,
    /// Ticks every set amount of time
    Tick,
}
