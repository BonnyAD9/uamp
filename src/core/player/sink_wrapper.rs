use std::{fmt::Debug, fs::File, path::Path, time::Duration};

use log::{error, warn};
use raplay::{
    CallbackInfo, CpalError, Sink, Timestamp,
    reexp::BuildStreamError,
    source::{Source, Symph, symph},
};
use ratag::{TagType, tag};

use crate::core::{
    Error, Result,
    library::{Library, LibraryUpdate, SongId},
    log_err,
    plugin::DecoderPlugin,
};

/// Wrapps the sink
pub struct SinkWrapper {
    /// The inner player
    sink: Sink,
    /// Configuration for symph sources
    symph: symph::Options,
    decoder_plugins: Vec<DecoderPlugin>,
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
            decoder_plugins: vec![],
        }
    }

    pub fn add_decoder_plugin(&mut self, plugin: DecoderPlugin) {
        self.decoder_plugins.push(plugin)
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
        let src = self.load_song(lib, id)?;
        self.unprefetch();
        self.load_inner(src, play)?;
        Ok(())
    }

    /// Prefetch the given song.
    pub fn prefetch(&mut self, lib: &mut Library, id: SongId) -> Result<()> {
        let src = self.load_song(lib, id)?;
        self.sink.prefetch(Some(src))?;
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
            self.load_song(lib, id)?
        };

        self.load_inner(src, play)?;
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
    pub fn get_timestamp(&self) -> Option<Timestamp> {
        match self.sink.get_timestamp() {
            Ok(ts) => Some(ts),
            Err(raplay::Error::NoSourceIsPlaying) => None,
            Err(e @ raplay::Error::Unsupported { .. }) => {
                warn!("Failed to get current timestamp: {e}");
                None
            }
            Err(e) => {
                dbg!(&e);
                error!("Failed to get current timestamp: {e}");
                None
            }
        }
    }

    /// Hard pauses the playback so that it doesn't just play silence.
    ///
    /// # Errors
    /// - Not supported by the device.
    pub fn hard_pause(&mut self) -> Result<()> {
        self.sink.hard_pause()?;
        Ok(())
    }

    fn load_song(
        &mut self,
        lib: &mut Library,
        id: SongId,
    ) -> Result<Box<dyn Source>> {
        let src = self.choose_decoder(lib[id].path())?;

        const SMALL_TIME: f64 = 0.1;

        if let Some(Timestamp { total, .. }) = src.get_time()
            && (lib[id].length().as_secs_f64() - total.as_secs_f64()).abs()
                > SMALL_TIME
        {
            lib[id].set_length(total);
            lib.update(LibraryUpdate::Metadata);
        }

        Ok(src)
    }

    fn choose_decoder(&self, p: impl AsRef<Path>) -> Result<Box<dyn Source>> {
        let mut probe = tag::Probe::top_level();
        log_err(
            "Failed to probe audio file.",
            ratag::read_tag_from_file(&p, &mut probe, &ratag::trap::Skip),
        );
        let skip_symph = probe.tags.iter().all(symphonia_unsupported);

        let mut errs = vec![];

        // We are almost sure that symphonia doesn't support this and will just
        // endlessly probe for mp3 sync.
        if !skip_symph {
            let file = File::open(p.as_ref())?;
            let err = match Symph::try_new(file.try_clone()?, &self.symph) {
                Ok(s) => return Ok(Box::new(s)),
                Err(e) => e,
            };
            errs.push(err.into());
        }

        for d in &self.decoder_plugins {
            match d.open(p.as_ref()) {
                Ok(d) => return Ok(d),
                Err(e) => errs.push(e),
            }
        }

        // Seems like no other decoder supports this. Actually try symphonia
        // even if it seems unlikely to work.
        if skip_symph {
            let file = File::open(p.as_ref())?;
            let err = match Symph::try_new(file.try_clone()?, &self.symph) {
                Ok(s) => return Ok(Box::new(s)),
                Err(e) => e,
            };
            errs.push(err.into());
        }

        Error::multiple(errs)?;
        unreachable!();
    }

    fn load_inner(&mut self, src: Box<dyn Source>, play: bool) -> Result<()> {
        let mut src = Some(src);
        match self.sink.try_load(&mut src, play) {
            e @ Err(raplay::Error::Cpal(CpalError::BuildStream(
                BuildStreamError::DeviceNotAvailable,
            ))) => {
                self.sink.restart_device(None)?;
                if src.is_some() {
                    self.sink.try_load(&mut src, play)
                } else {
                    e
                }
            }
            r => r,
        }?;
        Ok(())
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

fn symphonia_unsupported(typ: &TagType) -> bool {
    *typ == TagType::Asf
}
