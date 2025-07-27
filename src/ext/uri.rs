use std::path::Path;

/// Parses file uri and returns host and path to file.
#[cfg(windows)]
pub fn parse_file_uri(uri: &str) -> Option<(&str, &Path)> {
    uri.strip_prefix("file://")?
        .split_once('/')
        .map(|(host, path)| {
            if host.is_empty() {
                ("localhost", Path::new(path))
            } else {
                (host, Path::new(path))
            }
        })
}

/// Parses file uri and returns host and path to file.
#[cfg(not(windows))]
pub fn parse_file_uri(mut uri: &str) -> Option<(&str, &Path)> {
    uri = uri.strip_prefix("file://")?;
    uri.find('/').map(|idx| {
        if idx == 0 {
            ("localhost", Path::new(uri))
        } else {
            (&uri[..idx], Path::new(&uri[idx..]))
        }
    })
}

/// Creates file uri from path. Host may be empty for local files. The path
/// should be canonical.
#[cfg(windows)]
pub fn get_file_uri(mut host: &str, path: impl AsRef<Path>) -> String {
    if host == "localhost" {
        host = "";
    }
    format!("file://{host}/{}", path.as_ref().display())
}

/// Creates file uri from path. Host may be empty for local files. The path
/// should be canonical.
#[cfg(not(windows))]
pub fn get_file_uri(mut host: &str, path: impl AsRef<Path>) -> String {
    if host == "localhost" {
        host = "";
    }
    format!("file://{host}{}", path.as_ref().display())
}
