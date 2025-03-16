use std::{fmt::Debug, fs::File, time::Duration};

use log::warn;
use raplay::{
    CallbackInfo, Sink, Timestamp,
    source::{Source, Symph, symph},
};

use crate::core::{
    Result,
    library::{Library, LibraryUpdate, SongId},
};

/// Wrapps the sink
pub struct SinkWrapper {
    /// The inner player
    sink: Sink,
    /// Configuration for symph sources
    symph: symph::Options,
}

impl SinkWrapper {
    /// Create new [`SinkWrapper`]
    pub fn new() -> Self {
        let sink = Sink::default();
        sink.on_err_callback(Box::new(|e| warn!("Error in playback: {e}")))
            .expect("Failed to set sink error callback: ");

        // XXX: Make this configurable?
        sink.prefetch_notify(Duration::from_secs(1))
            .expect("Failed to set prefetch time on sink.");

        Self {
            sink,
            symph: symph::Options::default(),
        }
    }

    /// Plays the given song.
    ///
    /// # Errors
    /// - song fails to load.
    /// - synchronization, shouldn't happen.
    pub fn load(
        &mut self,
        lib: &mut Library,
        id: SongId,
        play: bool,
    ) -> Result<()> {
        let src = self.load_symph(lib, id)?;
        self.unprefetch();
        self.sink.load(Box::new(src), play)?;
        Ok(())
    }

    /// Prefetch the given song.
    pub fn prefetch(&mut self, lib: &mut Library, id: SongId) -> Result<()> {
        let src = self.load_symph(lib, id)?;
        self.sink.prefetch(Some(Box::new(src)))?;
        Ok(())
    }

    /// true - Send prefetch notification even if it has already been sent.
    ///
    /// false - Don't sent prefetch notification for the current source.
    pub fn do_prefetch_notify(&self, val: bool) {
        self.sink.do_prefetch_notify(val);
    }

    /// If there is prefetched song, load it, otherwise load the given song.
    pub fn load_or_prefetched(
        &mut self,
        lib: &mut Library,
        id: SongId,
        play: bool,
    ) -> Result<()> {
        let src = self
            .sink
            .prefetch(None)
            .expect("Failed to retrieve prefetched source.");

        let src = if let Some(src) = src {
            src
        } else {
            Box::new(self.load_symph(lib, id)?)
        };

        self.sink.load(src, play)?;
        Ok(())
    }

    pub fn unprefetch(&self) {
        self.sink
            .prefetch(None)
            .expect("Failed to remove prefetched song");
    }

    /// Sets the play state.
    ///
    /// # Panics
    /// - Synchronization problems.
    pub fn play(&mut self, play: bool) {
        self.sink.play(play).expect("Failed to play: ");
    }

    /// Sets callback for when a song ends.
    ///
    /// # Panics
    /// - Synchronization problems.
    pub fn on_callback<F>(&mut self, f: F)
    where
        F: Fn(CallbackInfo) + Send + 'static,
        &'static F: std::marker::Send,
    {
        self.sink
            .on_callback(Box::new(f))
            .expect("Failed to set sink callback: ");
    }

    /// Sets the playback volume.
    ///
    /// # Panics
    /// - Synchronization problems.
    pub fn set_volume(&mut self, volume: f32) {
        self.sink
            .volume(volume * volume)
            .expect("Failed to set volume: ");
    }

    /// Seeks to the given position.
    ///
    /// # Errors
    /// - Nothing is playing.
    /// - It is unsupported.
    /// - Synchronization problems (shouldn't happen).
    pub fn seek_to(&mut self, pos: Duration) -> Result<Timestamp> {
        Ok(self.sink.seek_to(pos)?)
    }

    /// Seeks by the given duration.
    ///
    /// # Errors
    /// - Nothing is playing.
    /// - It is unsupported.
    /// - Synchronization problems (shouldn't happen).
    pub fn seek_by(
        &mut self,
        time: Duration,
        forward: bool,
    ) -> Result<Timestamp> {
        Ok(self.sink.seek_by(time, forward)?)
    }

    /// Sets the fade play/pause duration in seconds.
    ///
    /// # Panics
    /// - Synchronization problems.
    pub fn fade_play_pause(&mut self, t: Duration) {
        self.sink
            .set_fade_len(t)
            .expect("Failed to set fade length: ");
    }

    /// Enables/disables gapless playback (applies only for the following calls
    /// to load).
    pub fn set_gapless(&mut self, v: bool) {
        self.symph.format.enable_gapless = v;
    }

    /// Gets the current timestamp of the playing source.
    ///
    /// # Errors
    /// - Nothing is playing.
    /// - It is unsupported.
    /// - Synchronization problems (shouldn't happen).
    pub fn get_timestamp(&self) -> Result<Timestamp> {
        Ok(self.sink.get_timestamp()?)
    }

    /// Hard pauses the playback so that it doesn't just play silence.
    ///
    /// # Errors
    /// - Not supported by the device.
    pub fn hard_pause(&mut self) -> Result<()> {
        self.sink.hard_pause()?;
        Ok(())
    }

    fn load_symph(&mut self, lib: &mut Library, id: SongId) -> Result<Symph> {
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

        Ok(src)
    }
}

impl Debug for SinkWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SinkWrapper")
            .field("sink", &self.sink)
            .field("symph", &())
            .finish()
    }
}
