use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct TimeStamp {
    pub current: Duration,
    pub total: Duration,
}

impl TimeStamp {
    pub fn new(current: Duration, total: Duration) -> Self {
        Self { current, total }
    }
}
