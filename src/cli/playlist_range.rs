use std::str::FromStr;

use pareg::{ArgError, ArgInto, FromArgStr};

#[derive(Debug, Clone, Copy)]
pub struct PlaylistRange(pub usize, pub usize);

impl FromStr for PlaylistRange {
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((s, e)) = s.split_once("..") {
            let a = s
                .arg_into::<Option<isize>>()
                .map_err(|e| e.shift_span(0, s.into()))?
                .unwrap_or_default()
                .unsigned_abs();
            let b = e
                .arg_into::<Option<usize>>()
                .map_err(|e| e.shift_span(s.len() + 2, s.into()))?
                .unwrap_or_default();
            Ok(Self(a, b))
        } else {
            Ok(Self(0, s.arg_into()?))
        }
    }
}

impl FromArgStr for PlaylistRange {}
