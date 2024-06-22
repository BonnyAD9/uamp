use std::{iter, time::Duration};

use itertools::Itertools;

use crate::core::{Error, Result};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

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
pub fn str_to_duration(s: &str) -> Result<Duration> {
    // Number of seconds in the time frame
    const MIN: u64 = 60;
    const HOUR: u64 = 60 * MIN;
    const DAY: u64 = 24 * HOUR;

    if s.is_empty() {
        return Err(Error::FailedToParse("Duration: String is empty."));
    }

    let r = s.split('d').collect_vec();
    let (d, hmsn) = match r.len() {
        1 => ("", r[0]),
        2 => (r[0], r[1]),
        _ => return Err(Error::InvalidValue("Duration: Too many 'd'.")),
    };

    let r = hmsn.split(':').collect_vec();
    let (h, m, sn) = match r.len() {
        1 => ("", "", r[0]),
        2 => ("", r[0], r[1]),
        3 => (r[0], r[1], r[2]),
        _ => return Err(Error::InvalidValue("Duration: Too many ':'.")),
    };

    let r = sn.split('.').collect_vec();
    let (s, mut n) = match (r.len(), sn.chars().next()) {
        (2, _) => (r[0], r[1]),
        (1, Some('.')) => ("", r[0]),
        (1, _) => (r[0], ""),
        (0, _) => ("", ""),
        _ => return Err(Error::InvalidValue("Duration: Too many '.'.")),
    };

    let mut res = Duration::ZERO;

    if !d.is_empty() {
        res += Duration::from_secs(d.parse::<u64>()? * DAY);
    }
    if !h.is_empty() {
        res += Duration::from_secs(h.parse::<u64>()? * HOUR);
    }
    if !m.is_empty() {
        res += Duration::from_secs(m.parse::<u64>()? * MIN);
    }
    if !s.is_empty() {
        res += Duration::from_secs(s.parse::<u64>()?);
    }
    if !n.is_empty() {
        let mut of = 0;
        if n.len() > 9 {
            let c = &n[9..10];
            if c.parse::<u64>()? >= 5 {
                of += 1;
            }
            n = &n[..9];
        }
        let p = 10u64.pow(9u32.saturating_sub(n.len() as u32));
        res += Duration::from_nanos(n.parse::<u64>()? * p + of)
    }

    Ok(res)
}
