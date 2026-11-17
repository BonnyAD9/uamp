use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use crate::core::{Error, Result, library::Song};

pub fn export_any_m3u_to_file<S: AsRef<Song>>(
    path: impl AsRef<Path>,
    songs: impl IntoIterator<Item = S>,
    utf8: bool,
) -> Result<()> {
    let path = path.as_ref();
    let f = File::create(path).map_err(|e| {
        Error::new(e).msg(format!(
            "Failed to open file `{}` for exporting m3u.",
            path.display()
        ))
    })?;

    export_any_m3u(BufWriter::new(f), songs, utf8).map_err(|e| {
        Error::new(e)
            .msg(format!("Failed to export m3u to `{}`.", path.display()))
    })
}

pub fn export_any_m3u<S: AsRef<Song>>(
    mut out: impl Write,
    songs: impl IntoIterator<Item = S>,
    utf8: bool,
) -> Result<(), std::io::Error> {
    for s in songs {
        if utf8 {
            writeln!(out, "{}", s.as_ref().path().display())?;
        } else {
            out.write_all(s.as_ref().path().as_os_str().as_encoded_bytes())?;
            out.write_all(b"\n")?;
        }
    }
    Ok(())
}
