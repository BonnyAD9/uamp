use std::io::{self, IsTerminal};

use bitflags::bitflags;

bitflags! {
    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    pub struct ErrCtxFlags: u64 {
        const COLOR_MODE = 0x3;
        const COLOR_NEVER = 0x0;
        const COLOR_STDIN = 0x1;
        const COLOR_STDERR = 0x2;
        const COLOR_ALWAYS = 0x3;

        const SHOW_ERR = 0x4;
        const INNER_FIRST = 0x8;
    }
}

impl ErrCtxFlags {
    pub fn use_color(&self) -> bool {
        match *self & Self::COLOR_MODE {
            Self::COLOR_NEVER => false,
            Self::COLOR_ALWAYS => true,
            Self::COLOR_STDIN => io::stdin().is_terminal(),
            Self::COLOR_STDERR => io::stderr().is_terminal(),
            _ => unreachable!(),
        }
    }

    pub fn set_color(&mut self, c: Self) {
        *self = (*self & !Self::COLOR_MODE) | (c & Self::COLOR_MODE);
    }
}

impl Default for ErrCtxFlags {
    fn default() -> Self {
        Self::COLOR_STDERR
    }
}
