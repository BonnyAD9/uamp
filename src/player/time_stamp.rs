use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Timestamp of a source
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimeStamp {
    /// The current positoin
    pub current: Duration,
    /// The total length
    pub total: Duration,
}

impl TimeStamp {
    /// Creates new timestamp
    pub fn new(current: Duration, total: Duration) -> Self {
        Self { current, total }
    }
}
