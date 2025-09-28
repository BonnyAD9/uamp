use bitflags::bitflags;

bitflags! {
    pub struct Change: u64 {
        const LIBRARY_PATH = 0x1;
        const PLAYER_PATH = 0x2;
        const SEARCH_PATHS = 0x4;
        const AUDIO_EXTENSIONS = 0x8;
        const RECURSIVE_SEARCH = 0x10;
        const SERVER_ADDRESS = 0x20;
        const PORT = 0x40;
        const SKIN = 0x80;
        const ENABLE_SERVER = 0x100;
        const AUTO_RESTART = 0x200;
        const SYSTEM_PLAYER = 0x400;
        const CACHE_PATH = 0x800;
        const FADE_PLAY_PAUSE = 0x1000;
        const GAPLESS = 0x2000;
    }
}

impl Change {
    pub fn add_change(&mut self, ch: Self) {
        *self |= ch;
    }
}
