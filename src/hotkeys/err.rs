use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unknown key '{0}'")]
    UnknownKey(String),
    #[error("There was multiple keys, you can have multiple modifiers, but only one key")]
    MultipleKeys,
    #[error("You must have at least one key")]
    NoKey,
    #[error(transparent)]
    GlobalHotKey(#[from] global_hotkey::Error),
}
