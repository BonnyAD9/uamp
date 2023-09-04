use std::time::Duration;

/// Timestamp of a source
#[derive(Debug, Clone, Copy)]
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
