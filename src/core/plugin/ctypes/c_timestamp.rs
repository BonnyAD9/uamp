use raplay::Timestamp;

use crate::core::plugin::ctypes::CDuration;

#[repr(C)]
#[derive(Debug)]
pub struct CTimestamp {
    current: CDuration,
    total: CDuration,
}

impl From<CTimestamp> for Timestamp {
    fn from(value: CTimestamp) -> Self {
        Self {
            current: value.current.into(),
            total: value.total.into(),
        }
    }
}
