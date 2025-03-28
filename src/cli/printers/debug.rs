use std::time::Instant;

use crate::{
    cli::printers::ser,
    core::{library::Song, messenger::Info},
};

pub fn info(info: &Info) {
    println!("{:#?}", ser::Info::new(info));
}

pub fn song_list(songs: &[Song], send_time: Instant) {
    println!("{:#?}", ser::SongList::new(songs, send_time));
}
