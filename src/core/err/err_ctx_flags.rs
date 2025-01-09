use std::io::{self, IsTerminal};

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct ErrCtxFlags: u64 {
        const COLOR_MODE = 0x3;
        const COLOR_NEVER = 0x0;
        const COLOR_STDIN = 0x1;
        const COLOR_STDERR = 0x2;
        const COLOR_ALWAYS = 0x3;

        const SHOW_ERR = 0x4;
        const INNER_FIRST = 0x8;

        const SEVERITY = 0x30;
        const SEVERITY_WARN = 0x10;
        const SEVERITY_ERROR = 0x20;
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

    pub fn set_masked(&mut self, mask: Self, value: Self) {
        *self = (*self & !mask) | (value & mask);
    }

    pub fn set_color(&mut self, c: Self) {
        self.set_masked(Self::COLOR_MODE, c);
    }

    pub fn set_severity(&mut self, c: Self) {
        self.set_masked(Self::SEVERITY, c);
    }
}

impl Default for ErrCtxFlags {
    fn default() -> Self {
        Self::COLOR_STDERR | Self::SHOW_ERR | Self::SEVERITY_ERROR
    }
}
