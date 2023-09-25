use std::{fs::File, time::Duration};

use raplay::{
    source::{symph::SymphOptions, Source, Symph},
    CallbackInfo, Sink, Timestamp,
};

use crate::{
    core::Result,
    library::{Library, LibraryUpdate, SongId},
};

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

        if let Some(Timestamp { total, .. }) = src.get_time() {
            if (lib[id].length().as_secs_f64() - total.as_secs_f64()).abs()
                > SMALL_TIME
            {
                lib[id].set_length(total);
                lib.update(LibraryUpdate::Metadata);
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
    pub fn on_callback<F: Fn(CallbackInfo) + Send + 'static>(
        &mut self,
        f: F,
    ) -> Result<()>
    where
        &'static F: std::marker::Send,
    {
        self.sink.on_callback(Some(f))?;
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
    pub fn seek_to(&mut self, pos: Duration) -> Result<Timestamp> {
        Ok(self.sink.seek_to(pos)?)
    }

    /// Seeks by the given duration
    pub fn seek_by(
        &mut self,
        time: Duration,
        forward: bool,
    ) -> Result<Timestamp> {
        Ok(self.sink.seek_by(time, forward)?)
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
    pub fn get_timestamp(&self) -> Result<Timestamp> {
        Ok(self.sink.get_timestamp()?)
    }

    pub fn hard_pause(&mut self) -> Result<()> {
        self.sink.hard_pause()?;
        Ok(())
    }
}
