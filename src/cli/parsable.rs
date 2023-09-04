use anyhow::anyhow;
use itertools::Itertools;

use std::{str::FromStr, time::Duration as Dur};

/// Wraper around [`std::time::Duration`] that implements ToString and FromStr,
/// and can be converted to and from [`std::time::Duration`]
pub struct Duration(Dur);

impl From<Dur> for Duration {
    fn from(value: Dur) -> Self {
        Self(value)
    }
}

impl Into<Dur> for Duration {
    fn into(self) -> Dur {
        self.0
    }
}

impl FromStr for Duration {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let r = s.split(':').collect_vec();
        let (h, m, s) = match r.len() {
            0 => return Err(anyhow!("Imput string is empty.")),
            1 => ("", "", r[0]),
            2 => ("", r[0], r[1]),
            3 => (r[0], r[1], r[2]),
            _ => return Err(anyhow!("Too many colons")),
        };

        let mut res = if s.is_empty() {
            Dur::ZERO
        } else {
            Dur::from_secs_f32(f32::from_str(s)?)
        };

        res += if m.is_empty() {
            Dur::ZERO
        } else {
            Dur::from_secs(u64::from_str(m)? * 60)
        };

        res += if h.is_empty() {
            Dur::ZERO
        } else {
            Dur::from_secs(u64::from_str(h)? * 3600)
        };

        Ok(res.into())
    }
}
