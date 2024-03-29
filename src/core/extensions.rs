use std::{iter, path::PathBuf, str::FromStr, time::Duration};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to parse {0}")]
    FailedToParse(&'static str),
}

/// Converts duration to a human readable string. Set `trunc` to `true`,
/// if you want to truncate the seconds. Opposite of `str_to_duration`.
///
/// This conversion is losless if `trunc` is `false`.
pub fn duration_to_string(dur: Duration, trunc: bool) -> String {
    // Number of seconds in the time frame
    const MIN: u64 = 60;
    const HOUR: u64 = 60 * MIN;
    const DAY: u64 = 24 * HOUR;

    let mut secs = dur.as_secs();

    let d = secs / DAY;
    secs %= DAY;
    let h = secs / HOUR;
    secs %= HOUR;
    let m = secs / MIN;
    secs %= MIN;

    let mut res = String::new();
    if d != 0 {
        res += &format!("{d}d");
    }
    if h != 0 {
        res += &format!("{h:02}:");
    }
    if trunc {
        res += &format!("{m:02}:{secs:02}");
    } else {
        res += &format!("{m:02}:{secs:02}");
        if dur.subsec_nanos() != 0 {
            let s = dur.subsec_nanos().to_string();
            res.push('.');
            res.extend(iter::repeat('0').take(9 - s.len()));
            res += s.trim_end_matches('0');
        }
    }

    res
}

/// Parses string to duration. Opposite of `duration_to_string`.
/// This conversion is precise to single nanosecond - the precision of
/// Duration.
pub fn str_to_duration(s: &str) -> Option<Duration> {
    // Number of seconds in the time frame
    const MIN: u64 = 60;
    const HOUR: u64 = 60 * MIN;
    const DAY: u64 = 24 * HOUR;

    if s.is_empty() {
        return None;
    }

    let r = s.split('d').collect_vec();
    let (d, hmsn) = match r.len() {
        1 => ("", r[0]),
        2 => (r[0], r[1]),
        _ => return None,
    };

    let r = hmsn.split(':').collect_vec();
    let (h, m, sn) = match r.len() {
        1 => ("", "", r[0]),
        2 => ("", r[0], r[1]),
        3 => (r[0], r[1], r[2]),
        _ => return None,
    };

    let r = sn.split('.').collect_vec();
    let (s, mut n) = match (r.len(), sn.chars().next()) {
        (2, _) => (r[0], r[1]),
        (1, Some('.')) => ("", r[0]),
        (1, _) => (r[0], ""),
        (0, _) => ("", ""),
        _ => return None,
    };

    let mut res = Duration::ZERO;

    if !d.is_empty() {
        res += Duration::from_secs(d.parse::<u64>().ok()? * DAY);
    }
    if !h.is_empty() {
        res += Duration::from_secs(h.parse::<u64>().ok()? * HOUR);
    }
    if !m.is_empty() {
        res += Duration::from_secs(m.parse::<u64>().ok()? * MIN);
    }
    if !s.is_empty() {
        res += Duration::from_secs(s.parse::<u64>().ok()?);
    }
    if !n.is_empty() {
        let mut of = 0;
        if n.len() > 9 {
            let c = &n[9..10];
            if c.parse::<u64>().ok()? >= 5 {
                of += 1;
            }
            n = &n[..9];
        }
        let p = 10u64.pow(9u32.checked_sub(n.len() as u32).unwrap_or(0));
        res += Duration::from_nanos(n.parse::<u64>().ok()? * p + of)
    }

    Some(res)
}

pub trait Parses<T> {
    type Err;
    /// Parses this to the type T
    fn get_value(&self) -> Result<T, Self::Err>;
}

/// Implements the trait [`Parses`] for the given types. The types must
/// implement [`FromStr`]
macro_rules! impl_parses {
    ($($t:ty),+ $(,)?) => {
        $(
            impl Parses<$t> for str {
                type Err = <$t as FromStr>::Err;

                fn get_value(&self) -> Result<$t, Self::Err> {
                    self.parse()
                }
            }
        )+
    };
}

// Works only for types that implement FromStr
impl_parses!(
    u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize, f32, f64,
    bool
);

impl Parses<Duration> for str {
    type Err = ParseError;

    fn get_value(&self) -> Result<Duration, Self::Err> {
        str_to_duration(self).ok_or(ParseError::FailedToParse("Duration"))
    }
}

/// Wraps the type, used for custom trait implementations
pub struct Wrap<T>(pub T);

impl<T> Clone for Wrap<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Copy for Wrap<T> where T: Copy {}

impl<T> AsRef<T> for Wrap<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for Wrap<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> PartialEq for Wrap<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Wrap<T> where T: Eq {}

impl<T> PartialOrd for Wrap<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Ord for Wrap<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> From<T> for Wrap<T> {
    fn from(value: T) -> Self {
        Wrap(value)
    }
}

impl Serialize for Wrap<Duration> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        duration_to_string(self.0, false).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Wrap<Duration> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        str_to_duration(&s)
            .map(|d| Wrap(d))
            .ok_or(serde::de::Error::custom("Invalid duration format"))
    }
}

pub fn valid_filename<I>(s: I) -> PathBuf
where
    I: Iterator<Item = char>,
{
    PathBuf::from(
        s.map(|c: char| {
            if "<>:\"/\\|?*".contains(c) {
                '-'
            } else if c.is_ascii_control() {
                '-'
            } else {
                c
            }
        })
        .collect::<String>(),
    )
}
