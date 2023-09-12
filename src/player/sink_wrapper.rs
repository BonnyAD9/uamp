use std::{fs::File, time::Duration};

use raplay::{
    sink::CallbackInfo,
    source::{symph::SymphOptions, Source, Symph},
    Sink,
};

use crate::{
    core::Result,
    library::{Library, SongId},
};

use super::TimeStamp;

/// Wrapps the sink
pub struct SinkWrapper {
    /// The inner player
    sink: Sink,
    /// Configuration for symph sources
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
        lib: &mut Library,
        id: SongId,
        play: bool,
    ) -> Result<()> {
        let file = File::open(lib[id].path())?;
        let src = Symph::try_new(file, &self.symph)?;

        const SMALL_TIME: f64 = 0.1;

        if let Some((_, total)) = src.get_time() {
            if (lib[id].length().as_secs_f64() - total.as_secs_f64()).abs()
                > SMALL_TIME
            {
                lib[id].set_length(total);
            }
        }

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

    /// Seeks to the given position
    pub fn seek_to(&mut self, pos: Duration) -> Result<()> {
        self.sink.seek_to(pos)?;
        Ok(())
    }

    /// Sets the fade play/pause duration in seconds
    pub fn fade_play_pause(&mut self, t: Duration) -> Result<()> {
        self.sink.set_fade_len(t)?;
        Ok(())
    }

    /// Enables/disables gapless playback (applies only for the following calls
    /// to load)
    pub fn set_gapless(&mut self, v: bool) {
        self.symph.format.enable_gapless = v;
    }

    /// Gets the current timestamp of the playing source
    pub fn get_timestamp(&self) -> Result<TimeStamp> {
        Ok(self
            .sink
            .get_timestamp()
            .map(|(c, t)| TimeStamp::new(c, t))?)
    }
}
