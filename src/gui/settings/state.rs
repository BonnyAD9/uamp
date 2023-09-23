#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum Category {
    #[default]
    Library,
    Playback,
    Hotkeys,
    Server,
    Other,
}
