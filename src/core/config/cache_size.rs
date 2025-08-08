use std::fmt::Display;

use pareg::FromArg;

#[derive(Copy, Clone, PartialEq, Eq, Default, FromArg)]
pub enum CacheSize {
    #[default]
    Full = 0,
    #[arg("64")]
    S64 = 6,
    #[arg("128")]
    S128 = 7,
    #[arg("256")]
    S256 = 8,
}

impl CacheSize {
    pub fn size(&self) -> Option<usize> {
        if matches!(self, Self::Full) {
            None
        } else {
            Some(1 << *self as usize)
        }
    }

    pub fn for_size(s: usize) -> Self {
        match s {
            ..=64 => Self::S64,
            65..=128 => Self::S128,
            129..=256 => Self::S256,
            _ => Self::Full,
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
