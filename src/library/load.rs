use std::{thread::JoinHandle, time::Instant};

use super::Song;

pub struct LibraryLoad {
    pub handle: JoinHandle<LibraryLoadResult>,
    pub time_started: Instant,
}

pub struct LibraryLoadResult {
    pub new_song_vec: Option<Vec<Song>>,
}
