use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(
        "Expected more arguments{} (the last argument requires that more follow)",
        if let Some(i) = .0 { format!(" after '{}'", i) } else { "".to_owned() }
    )]
    UnexpectedEnd(Option<String>),
    #[error(
        "Failed to parse argument{}, the argument must be {typ}",
        if let Some(i) = .id { i.as_str() } else { "" }
    )]
    ParseError {
        id: Option<String>,
        typ: &'static str,
    },
    #[error("Unknown option{}", .0.as_ref().map(|i| i.as_str()).unwrap_or(""))]
    UnknownArgument(Option<String>),
    #[error(
        "Missing parameter{}",
        if let Some(i) = .0 { format!(" for argument '{}'", i) } else { "".to_owned() })]
    MissingParameter(Option<String>),
}
