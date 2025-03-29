use std::{fmt::Debug, time::Instant};

use pareg::FromArg;

use crate::core::{config::Config, library::Song, messenger::Info};

use super::{debug, json, pretty};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, FromArg)]
pub enum PrintStyle {
    #[default]
    Pretty,
    Debug,
    Json,
}

impl PrintStyle {
    pub fn info(&self, info: &Info, conf: &Config, color: bool, lmc: bool) {
        match self {
            PrintStyle::Pretty => pretty::info(info, conf, color, lmc),
            PrintStyle::Debug => debug::info(info),
            PrintStyle::Json => json::info(info),
        }
    }

    pub fn song_list(&self, songs: &[Song], color: bool, send_time: Instant) {
        match self {
            PrintStyle::Pretty => pretty::song_list(songs, color, send_time),
            PrintStyle::Debug => debug::song_list(songs, send_time),
            PrintStyle::Json => json::song_list(songs, send_time),
        }
    }
}
