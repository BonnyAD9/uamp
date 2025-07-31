use serde::{Deserialize, Serialize};

use crate::core::library::Song;

use super::Info;

#[derive(Debug, Serialize, Deserialize)]
pub enum RepMsg {
    Info(Box<Info>),
    Query(Vec<Song>),
    Error(String),
}
