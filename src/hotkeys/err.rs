use thiserror::Error;

/// Error used in the hotkeys module
#[derive(Error, Debug)]
pub enum Error {
    /// Unknown key when parsing
    #[error("Unknown key '{0}'")]
    UnknownKey(String),
    /// While parsing, there were multiple keys that are not modifiers
    #[error("There was multiple keys, you can have multiple modifiers, but only one key")]
    MultipleKeys,
    /// While parsing, there were only modifiers
    #[error("You must have at least one key")]
    NoKey,
    /// Error when registering global hotkey
    #[error(transparent)]
    GlobalHotKey(#[from] global_hotkey::Error),
}
