mod exim_format;
mod m3u;
mod pls;

use std::path::Path;

use crate::core::{Error, Result, library::Song};

pub use self::{exim_format::*, m3u::*, pls::*};

pub fn export_playlist_to_file<S: AsRef<Song>>(
    path: impl AsRef<Path>,
    songs: impl IntoIterator<Item = S>,
    format: Option<EximFormat>,
) -> Result<()> {
    let path = path.as_ref();
    let fmt = format
        .or_else(|| EximFormat::from_ext(path.extension()?.to_str()?))
        .ok_or_else(|| {
            Error::unknown_file_extension()
                .msg("Failed to select playlist format.")
        })?;
    match fmt {
        EximFormat::M3u => export_any_m3u_to_file(path, songs, false),
        EximFormat::M3u8 => export_any_m3u_to_file(path, songs, true),
        EximFormat::Pls => export_pls_to_file(path, songs),
    }
}
