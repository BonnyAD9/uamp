#[repr(i32)]
#[derive(Debug)]
#[allow(dead_code)] // It is constructed by external plugins
pub enum CErrorType {
    NoError = 0,
    Recoverable = 1,
    Fatal = 2,
}

impl CErrorType {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::NoError),
            1 => Some(Self::Recoverable),
            2 => Some(Self::Fatal),
            _ => None
        }
    }
}