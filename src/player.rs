use std::fs::File;

use eyre::Result;
use raplay::{
    sink::{CallbackInfo, Sink},
    source::symph::Symph,
};

use crate::library::Library;

pub struct Player {
    sink: Sink,
}

impl Player {
    pub fn try_new() -> Result<Self> {
        Ok(Player {
            sink: Sink::default_out()?,
        })
    }

    pub fn play(&mut self, lib: &Library, id: usize) -> Result<()> {
        let file = File::open(lib[id].path())?;
        let src = Symph::try_new(file)?;
        self.sink.load(src, true)
    }

    pub fn play_pause(&mut self, play: bool) -> Result<()> {
        self.sink.play(play)
    }

    pub fn on_song_end<F: FnMut() + Send + 'static>(
        &mut self,
        mut f: F,
    ) -> Result<()>
    where
        &'static F: std::marker::Send,
    {
        self.sink.on_callback(Some(move |cb| match cb {
            CallbackInfo::SourceEnded => f(),
            _ => {}
        }))
    }
}
