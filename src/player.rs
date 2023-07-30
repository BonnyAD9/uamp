use std::fs::File;

use eyre::Result;
use raplay::{sink::Sink, source::symph::Symph};

use crate::library::Library;

pub struct Player {
    current: Option<usize>,
    sink: Sink,
}

impl Player {
    pub fn try_new() -> Result<Self> {
        Ok(Player {
            current: None,
            sink: Sink::default_out()?,
        })
    }

    pub fn play(&mut self, lib: &Library, id: usize) -> Result<()> {
        let file = File::open(lib[id].path())?;
        let src = Symph::try_new(file)?;
        self.sink.load(src, true)
    }

    pub fn play_pause(&mut self) -> Result<()> {
        self.sink.play(!self.sink.is_playing()?)
    }
}
