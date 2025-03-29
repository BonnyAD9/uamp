use std::time::{Duration, Instant};

use serde::Serialize;

use crate::{
    core::{config::VERSION_STR, library::Song},
    ext::Wrap,
};

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct SongList<'a> {
    version: &'static str,
    time: Wrap<Duration>,
    songs: &'a [Song],
}

impl<'a> SongList<'a> {
    pub fn new(songs: &'a [Song], send_time: Instant) -> Self {
        Self {
            version: VERSION_STR,
            songs,
            time: Wrap(Instant::now() - send_time),
        }
    }
}
