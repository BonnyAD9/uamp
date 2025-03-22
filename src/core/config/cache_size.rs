use std::fmt::Display;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CacheSize {
    S128 = 7,
    S256 = 8,
}

impl CacheSize {
    pub fn size(&self) -> usize {
        1 << *self as usize
    }
}

impl Display for CacheSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.size())
    }
}
