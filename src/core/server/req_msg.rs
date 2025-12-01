use pareg::{ArgError, parse_arg};

use crate::{cli::PlaylistRange, core::query::Query};

/// Request someting from the other side.
#[derive(Debug)]
pub enum ReqMsg {
    /// Request the current playback info.
    Info(usize, usize),
    /// Query for songs
    Query(Query),
}

impl ReqMsg {
    pub fn from_kv(k: &str, v: &str) -> pareg::Result<Self> {
        match k {
            "info" | "nfo" | "show" => {
                let s =
                    parse_arg::<Option<_>>(v)?.unwrap_or(PlaylistRange(1, 3));
                Ok(ReqMsg::Info(s.0, s.1))
            }
            "query" | "list" | "l" => Ok(ReqMsg::Query(
                parse_arg::<Option<_>>(v)?.unwrap_or_default(),
            )),
            _ => ArgError::failed_to_parse("Invalid request type.", k).err(),
        }
    }
}
