use std::{fs::File, io::BufReader};

use eyre::Result;
use rodio::{Decoder, OutputStream, Sink};

use crate::library::Library;

pub struct Player {
    current: Option<usize>,
    sink: Sink,
    _stream: OutputStream,
}

impl Player {
    pub fn try_new() -> Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        Ok(Player {
            current: None,
            sink: Sink::try_new(&stream_handle)?,
            _stream: stream,
        })
    }

    pub fn play(&mut self, lib: &Library, id: usize) -> Result<()> {
        self.sink.stop();
        let file = BufReader::new(File::open(lib[id].path())?);
        self.current = Some(id);
        let source = Decoder::new(file)?;
        self.sink.append(source);
        self.sink.play();
        Ok(())
    }

    pub fn play_pause(&mut self) -> bool {
        if self.sink.is_paused() {
            self.sink.play();
            true
        } else {
            self.sink.pause();
            false
        }
    }
}
