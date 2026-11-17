use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use crate::core::{Error, Result, library::Song};

pub fn export_pls_to_file<S: AsRef<Song>>(
    path: impl AsRef<Path>,
    songs: impl IntoIterator<Item = S>,
) -> Result<()> {
    let path = path.as_ref();
    let f = File::create(path).map_err(|e| {
        Error::new(e).msg(format!(
            "Failed to open file `{}` for exporting pls.",
            path.display(),
        ))
    })?;

    export_pls(BufWriter::new(f), songs).map_err(|e| {
        Error::new(e)
            .msg(format!("Failed to export pls to `{}`.", path.display()))
    })
}

pub fn export_pls<S: AsRef<Song>>(
    mut out: impl Write,
    songs: impl IntoIterator<Item = S>,
) -> Result<(), std::io::Error> {
    out.write_all(b"[playlist]\n")?;

    let mut i = 0;
    for s in songs {
        i += 1;
        let s = s.as_ref();

        write!(out, "File{i}=")?;
        out.write_all(s.path().as_os_str().as_encoded_bytes())?;

        if let Some(t) = s.title() {
            write!(out, "\nTitle{i}=")?;

            let mut artists = s.artists().iter();
            if let Some(a) = artists.next() {
                write!(out, "{a}")?;
            }
            for a in artists {
                write!(out, ", {a}")?;
            }

            if !s.artists().is_empty() {
                write!(out, " - ")?;
            }
            writeln!(out, "{t}")?;
        }

        if let Some(l) = s.length() {
            writeln!(out, "Length{i}={}", l.as_secs())?;
        }
    }

    writeln!(out, "NumberOfEntries={i}\nVersion=2")
}
