use std::time::Duration;

#[repr(C)]
#[derive(Debug)]
pub struct CDuration {
    secs: u64,
    nanos: u32,
}

impl From<Duration> for CDuration {
    fn from(value: Duration) -> Self {
        Self {
            secs: value.as_secs(),
            nanos: value.subsec_nanos(),
        }
    }
}

impl From<CDuration> for Duration {
    fn from(value: CDuration) -> Self {
        Duration::from_secs(value.secs)
            + Duration::from_nanos(value.nanos as u64)
    }
}
