use thiserror::Error;

/// Result with the CLI error
pub type Result<T> = std::result::Result<T, Error>;

/// Error from CLI
#[derive(Error, Debug)]
pub enum Error {
    /// Expected more arguments. Contains the las argument
    #[error(
        "Expected more arguments{} (the last argument requires that more follow)",
        if let Some(i) = .0 { format!(" after '{}'", i) } else { "".to_owned() }
    )]
    UnexpectedEnd(Option<String>),
    /// Failed to parse artument.
    #[error(
        "Failed to parse argument {}, the argument must be {typ}",
        if let Some(i) = .id { i.as_str() } else { "" }
    )]
    Parse {
        /// Name of the argument that failed to parse
        id: Option<String>,
        /// What was expected
        typ: &'static str,
    },
    /// Unknown option in the argument list
    #[error("Unknown option {}", .0.as_ref().map(|i| i.as_str()).unwrap_or(""))]
    UnknownArgument(Option<String>),
    /// Missing immidiate parameter to a artument. Contains the name of the
    /// argument to which the parameter is missing
    #[error(
        "Missing parameter {}",
        if let Some(i) = .0 { format!(" for argument '{}'", i) } else { "".to_owned() })]
    MissingParameter(Option<String>),
}
