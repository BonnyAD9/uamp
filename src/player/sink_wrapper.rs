use std::{fs::File, time::Duration};

use raplay::{
    sink::CallbackInfo,
    source::{symph::SymphOptions, Symph},
    Sink,
};

use crate::{
    core::Result,
    library::{Library, SongId},
};

use super::TimeStamp;

/// Wrapps the sink
pub struct SinkWrapper {
    sink: Sink,
    symph: SymphOptions,
}

impl SinkWrapper {
    /// Create new [`SinkWrapper`]
    pub fn new() -> Self {
        Self {
            sink: Sink::default(),
            symph: SymphOptions::default(),
        }
    }

    /// Plays the given song
    ///
    /// # Errors
    /// - only with synchronization, shouldn't happen
    pub fn load(
        &mut self,
        lib: &Library,
        id: SongId,
        play: bool,
    ) -> Result<()> {
        let file = File::open(lib[id].path())?;
        let src = Symph::try_new(file, &self.symph)?;
        self.sink.load(src, play)?;
        Ok(())
    }

    /// Sets the play state
    ///
    /// # Errors
    /// - only with synchronization, shouldn't happen
    pub fn play(&mut self, play: bool) -> Result<()> {
        self.sink.play(play)?;
        Ok(())
    }

    /// Sets callback for when a song ends
    ///
    /// # Errors
    /// - only with synchronization, shouldn't happen
    pub fn on_song_end<F: FnMut() + Send + 'static>(
        &mut self,
        mut f: F,
    ) -> Result<()>
    where
        &'static F: std::marker::Send,
    {
        self.sink.on_callback(Some(move |cb| match cb {
            CallbackInfo::SourceEnded => {
                f();
            }
            _ => {}
        }))?;
        Ok(())
    }

    /// Sets the playback volume
    ///
    /// # Errors
    /// - only with synchronization, shouldn't happen
    pub fn set_volume(&mut self, volume: f32) -> Result<()> {
        self.sink.volume(volume * volume)?;
        Ok(())
    }

    pub fn seek_to(&mut self, pos: Duration) -> Result<()> {
        self.sink.seek_to(pos)?;
        Ok(())
    }

    pub fn fade_play_pause(&mut self, secs: f32) -> Result<()> {
        self.sink.set_fade_len(Duration::from_secs_f32(secs))?;
        Ok(())
    }

    pub fn set_gapless(&mut self, v: bool) {
        self.symph.format.enable_gapless = v;
    }

    pub fn get_timestamp(&self) -> Result<TimeStamp> {
        Ok(self
            .sink
            .get_timestamp()
            .map(|(c, t)| TimeStamp::new(c, t))?)
    }
}
