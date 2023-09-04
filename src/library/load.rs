use std::{thread::JoinHandle, time::Instant};

use super::Song;

/// Contains metadata about library load on another thread
pub struct LibraryLoad {
    pub handle: JoinHandle<LibraryLoadResult>,
    pub time_started: Instant,
}

/// Result of library load on another thread
pub struct LibraryLoadResult {
    pub new_song_vec: Option<Vec<Song>>,
}
