use std::{fs::File, sync::Arc};

use eyre::Result;
use log::error;
use raplay::{
    sink::{CallbackInfo, Sink},
    source::symph::Symph,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    library::{Library, SongId},
    uamp_app::UampMessage,
    wid::Command,
};

struct SinkWrapper {
    sink: Sink,
}

impl SinkWrapper {
    pub fn try_new() -> Result<Self> {
        Ok(Self {
            sink: Sink::default_out()?,
        })
    }

    pub fn play(
        &mut self,
        lib: &Library,
        id: SongId,
        play: bool,
    ) -> Result<()> {
        let file = File::open(lib[id].path())?;
        let src = Symph::try_new(file)?;
        self.sink.load(src, play)
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

#[derive(Clone, Copy, Debug)]
pub enum PlayerMessage {
    SongEnd,
}

#[derive(Clone, Copy)]
pub enum Playback {
    Stopped,
    Playing,
    Paused,
}

pub struct Player {
    inner: Option<SinkWrapper>,
    state: Playback,
    playlist: Arc<[SongId]>,
    current: Option<usize>,
}

impl Player {
    pub fn new(sender: Arc<UnboundedSender<UampMessage>>) -> Self {
        let inner = match SinkWrapper::try_new() {
            Err(e) => {
                error!("Failed to create player: {e}");
                None
            }
            Ok(mut i) => {
                _ = i.on_song_end(move || {
                    _ = sender
                        .send(UampMessage::Player(PlayerMessage::SongEnd));
                });
                Some(i)
            }
        };

        Self {
            inner,
            state: Playback::Stopped,
            playlist: [][..].into(),
            current: None,
        }
    }

    pub fn event(&mut self, lib: &Library, msg: PlayerMessage) -> Command {
        match msg {
            PlayerMessage::SongEnd => self.play_next(lib),
        }
        Command::none()
    }

    fn try_get_player(&mut self) {
        if self.inner.is_none() {
            self.inner = SinkWrapper::try_new().ok();
        }
    }

    pub fn play(&mut self, play: bool) {
        self.try_get_player();
        if let Some(p) = self.inner.as_mut() {
            self.state = if play {
                Playback::Playing
            } else {
                Playback::Paused
            };

            _ = p.play_pause(play);
        }
    }

    pub fn try_load(
        &mut self,
        lib: &Library,
        index: Option<usize>,
        play: bool,
    ) {
        match index {
            Some(i) if i < self.playlist.len() => self.load(lib, i, play),
            _ => {
                self.current = None;
                self.state = Playback::Stopped;
            }
        }
    }

    fn load(&mut self, lib: &Library, index: usize, play: bool) {
        self.current = Some(index);

        self.try_get_player();
        let inner = match self.inner.as_mut() {
            Some(i) => i,
            None => {
                self.state = Playback::Stopped;
                return;
            }
        };

        self.state = if play {
            Playback::Playing
        } else {
            Playback::Paused
        };

        _ = inner.play(lib, self.playlist[index], play);
    }

    pub fn play_pause(&mut self) {
        match self.state {
            Playback::Stopped => {}
            Playback::Playing => {
                self.play(false);
            }
            Playback::Paused => {
                self.play(true);
            }
        }
    }

    pub fn play_playlist(
        &mut self,
        lib: &Library,
        songs: Arc<[SongId]>,
        index: Option<usize>,
        play: bool,
    ) {
        self.playlist = songs;
        self.try_load(lib, index, play);
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.state, Playback::Playing)
    }

    pub fn now_playing(&self) -> Option<SongId> {
        self.current.map(|i| self.playlist[i])
    }

    pub fn play_next(&mut self, lib: &Library) {
        match self.state {
            Playback::Stopped => {}
            Playback::Playing => {
                self.try_load(lib, self.current.map(|i| i + 1), true);
            }
            Playback::Paused => {
                self.try_load(lib, self.current.map(|i| i + 1), false)
            }
        }
    }
}
