use bitflags::bitflags;

bitflags! {
    #[derive(Default, Debug)]
    pub struct Job: u32 {
        const LIBRARY_LOAD = 1;
        const SERVER = 2;
        const LIBRARY_SAVE = 4;
        const SYSTEM_PLAYER = 8;

        const NO_CLOSE = 7;
    }
}
