use std::fmt::Display;

use const_format::{concatc, str_splice, str_split};
use serde::{Deserialize, Serialize};

/// Version of uamp as string.
pub const VERSION_STR: &str = {
    if VERSION_COMMIT.is_some() {
        const COMMIT: &str = if let Some(commit) = VERSION_COMMIT {
            commit
        } else {
            "unknown-commit" // Unreachable
        };
        const COMMIT_SHORT: &str = str_splice!(COMMIT, ..8, "").removed;
        concatc!(VERSION_NUMBER, "-", COMMIT_SHORT)
    } else {
        VERSION_NUMBER
    }
};

/// Version number of uamp
pub const VERSION_NUMBER: &str = {
    let v = option_env!("CARGO_PKG_VERSION");
    if let Some(v) = v { v } else { "unknown" }
};

/// Commit of uamp. Not present in releases.
pub const VERSION_COMMIT: Option<&str> = option_env!("UAMP_VERSION_COMMIT");

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct Version(pub usize, pub usize, pub usize);

/// Version of uamp as major minor and patch. (0, 1, 2)
pub const VERSION: Option<Version> = {
    let comp = str_split!(VERSION_NUMBER, '.');
    if comp.len() != 3 {
        None
    } else {
        let a = parse_num(comp[0]);
        let b = parse_num(comp[1]);
        let c = parse_num(comp[2]);
        if let Some(a) = a
            && let Some(b) = b
            && let Some(c) = c
        {
            Some(Version(a, b, c))
        } else {
            None
        }
    }
};

const fn parse_num(s: &str) -> Option<usize> {
    let mut res = 0;
    let b = s.as_bytes();
    let mut i = 0;
    loop {
        if i >= b.len() {
            break;
        }

        if !b[i].is_ascii_digit() {
            return None;
        }
        res = res * 10 + (b[i] - b'0') as usize;
        i += 1;
    }

    Some(res)
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}
