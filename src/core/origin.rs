#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Copy)]
pub enum Origin {
    Remote,
    Local,
    Internal,
}

impl Origin {
    pub fn remote() -> Self {
        Self::Remote
    }
}
