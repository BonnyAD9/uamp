use std::{
    fmt::{Debug, Display},
    str::FromStr,
    time::Duration,
};

use pareg::FromArgStr;
use serde::{Deserialize, Serialize};

use crate::core::Error;

use super::{duration_to_string, str_to_duration};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Generic wrapper struct. Used to implement foreign traits for foreign types.
pub struct Wrap<T>(pub T);

impl<T> Debug for Wrap<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Wrap").field(&self.0).finish()
    }
}

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
            .map(Wrap)
            .map_err(|_| serde::de::Error::custom("Invalid duration format"))
    }
}

impl FromStr for Wrap<Duration> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        str_to_duration(s).map(|a| a.into())
    }
}

impl FromArgStr for Wrap<Duration> {}

impl Display for Wrap<Duration> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&duration_to_string(self.0, false))
    }
}
