use std::{io, time::Instant};

use crate::core::{library::Song, server::Info};

use super::ser;

pub fn info(info: &Info) {
    _ = serde_json::to_writer_pretty(
        io::stdout().lock(),
        &ser::Info::new(info),
    );
    println!();
}

pub fn song_list(songs: &[Song], send_time: Instant) {
    _ = serde_json::to_writer_pretty(
        io::stdout().lock(),
        &ser::SongList::new(songs, send_time),
    );
    println!();
}
