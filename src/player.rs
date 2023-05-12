use std::{fs::File, io::BufReader};

use eyre::Result;
use rodio::{Decoder, OutputStream, Sink};

use crate::song::Song;

pub struct Player {
    current: Option<Song>,
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

    pub fn play(&mut self, song: Song) -> Result<()> {
        self.sink.stop();
        let file = BufReader::new(File::open(song.path())?);
        self.current = Some(song);
        let source = Decoder::new(file)?;
        self.sink.append(source);
        self.sink.play();
        Ok(())
    }
}
