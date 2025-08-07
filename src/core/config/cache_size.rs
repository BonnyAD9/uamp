use std::fmt::Display;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CacheSize {
    S128 = 7,
    S256 = 8,
    Full = 0,
}

impl CacheSize {
    pub fn size(&self) -> Option<usize> {
        if matches!(self, Self::Full) {
            None
        } else {
            Some(1 << *self as usize)
        }
    }
}

impl Display for CacheSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(s) = self.size() {
            write!(f, "{s}")
        } else {
            f.write_str("full")
        }
    }
}
