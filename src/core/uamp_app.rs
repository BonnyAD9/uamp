use std::{fs, net::TcpListener, path::Path, process, time::Instant};

use futures::{channel::mpsc::UnboundedSender, StreamExt};
use log::{error, warn};
use notify::{INotifyWatcher, Watcher};
use signal_hook_async_std::Signals;

use crate::{
    core::{
        messenger::{self, Messenger, MsgMessage},
        msg::{ControlMsg, Msg},
        Error, Result,
    },
    env::{AppCtrl, MsgGen},
};

use super::{
    config::{default_config_path, Config, ConfigMsg},
    library::Library,
    player::Player,
    TaskMsg, TaskType,
};

/// The uamp app state
pub struct UampApp {
    /// The configuration
    pub config: Config,
    /// The song library
    pub library: Library,
    /// The song player
    pub player: Player,

    /// Sender for async messages to be synchronized with the main message
    /// handler
    pub sender: UnboundedSender<Msg>,

    /// Messages that can be processed only if there are no running processes
    pub pending_close: bool,

    pub hard_pause_at: Option<Instant>,

    /// When was last save
    pub last_save: Instant,

    pub last_prev: Instant,

    _config_watch: Option<INotifyWatcher>,
}

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

impl UampApp {
    pub fn new(
        conf: Config,
        ctrl: &mut AppCtrl,
        sender: UnboundedSender<Msg>,
    ) -> Result<Self> {
        let mut lib = Library::from_config(&conf);

        if conf.update_library_on_start() {
            lib.start_get_new_songs(&conf, ctrl, Default::default())?;
        }

        let mut player = Player::from_config(&mut lib, sender.clone(), &conf);
        player.load_config(&conf);
        player.remove_deleted(&lib);

        if conf.enable_server() || conf.force_server {
            Self::start_server(&conf, ctrl, sender.clone())?;
        }

        Self::signal_task(ctrl)?;

        if conf.play_on_start() {
            if let Err(e) = sender.unbounded_send(Msg::Control(
                ControlMsg::PlayPause(Some(true)),
            )) {
                error!("Failed to send message to play on startup: {e}");
            }
        }

        let config_watch = if let Some(path) = &conf.config_path {
            match Self::watch_config_task(sender.clone(), path.as_path()) {
                Ok(r) => Some(r),
                Err(e) => {
                    error!("Failed to watch config file: {e}");
                    None
                }
            }
        } else {
            None
        };

        Ok(UampApp {
            config: conf,
            library: lib,
            player,

            sender,

            pending_close: false,

            last_save: Instant::now(),
            last_prev: Instant::now(),

            hard_pause_at: None,

            _config_watch: config_watch,
        })
    }

    /// Saves all the data that is saved by uamp
    pub fn save_all(&mut self, closing: bool, ctrl: &mut AppCtrl) {
        match self.library.start_to_default_json(
            &self.config,
            ctrl,
            &mut self.player,
        ) {
            Err(Error::InvalidOperation(_)) => {}
            Err(e) => error!("Failed to start library save: {e}"),
            _ => {}
        }
        if let Err(e) = self.player.save_to_default_json(closing, &self.config)
        {
            error!("Failed to save play state: {e}");
        }
        if let Err(e) = self.config.to_default_json() {
            error!("Failed to save config: {e}");
        }

        self.last_save = Instant::now();
    }

    pub fn task_end(&mut self, ctrl: &mut AppCtrl, task_res: TaskMsg) {
        match task_res {
            TaskMsg::Server(Err(e)) => {
                error!("Server unexpectedly ended: {e}");
            }
            TaskMsg::Server(Ok(_)) => {
                if self.config.enable_server() || self.config.force_server {
                    if let Err(e) = Self::start_server(
                        &self.config,
                        ctrl,
                        self.sender.clone(),
                    ) {
                        error!("Failed to restart server: {e}");
                    }
                }
            }
            TaskMsg::LibraryLoad(res) => {
                self.finish_library_load(ctrl, res);
            }
            TaskMsg::LibrarySave(res) => {
                self.finish_library_save_songs(res);
            }
        }
    }

    pub fn update(&mut self, ctrl: &mut AppCtrl, message: Msg) {
        let msg = match message {
            Msg::PlaySong(msg) => self.play_event(msg),
            Msg::Control(msg) => self.control_event(ctrl, msg),
            Msg::DataControl(msg) => self.data_control_event(ctrl, msg),
            Msg::Player(msg) => self.player_event(msg),
            Msg::Delegate(d) => d.update(self, ctrl),
            Msg::Config(msg) => self.config_event(ctrl, msg),
            Msg::Tick => None,
            Msg::None => None,
        };

        if let Some(msg) = msg {
            return self.update(ctrl, msg);
        }

        if self.pending_close && !ctrl.any_task(|t| t != TaskType::Server) {
            self.pending_close = false;
            return self.update(ctrl, Msg::Control(ControlMsg::Close));
        }

        let now = Instant::now();
        if let Some(t) = self.hard_pause_at {
            if t <= now {
                self.player.hard_pause();
                self.hard_pause_at = None;
            }
        }

        if self
            .config
            .save_timeout()
            .map(|t| now - self.last_save >= t.0)
            .unwrap_or_default()
        {
            self.save_all(false, ctrl);
        }

        let up = self.library_lib_update();
        self.player_lib_update(up);
    }

    /// Deletes old logs.
    pub fn delete_old_logs(&self) -> Result<()> {
        let dir = fs::read_dir(default_config_path().join("log"))?;

        for d in dir {
            let d = d?;
            let mt = d.metadata()?.modified()?;
            if mt.elapsed()? > self.config.delete_logs_after().0 {
                fs::remove_file(d.path())?;
            }
        }

        Ok(())
    }

    /// Starts the tcp server
    pub fn start_server(
        conf: &Config,
        ctrl: &mut AppCtrl,
        sender: UnboundedSender<Msg>,
    ) -> Result<()> {
        if ctrl.is_task_running(TaskType::Server) {
            return Err(Error::InvalidOperation("Server is already running"));
        }

        let listener = TcpListener::bind(format!(
            "{}:{}",
            conf.server_address(),
            conf.port()
        ))?;

        let task =
            move || TaskMsg::Server(Ok(Self::server_task(listener, sender)));
        ctrl.add_task(TaskType::Server, task);

        Ok(())
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl UampApp {
    fn signal_task(ctrl: &mut AppCtrl) -> Result<()> {
        let sig = Signals::new(signal_hook::consts::TERM_SIGNALS)?;

        let stream = MsgGen::new((sig, 0), |(mut sig, cnt)| async move {
            let Some(s) = sig.next().await else {
                return (Some((sig, cnt)), Msg::None);
            };

            if signal_hook::consts::TERM_SIGNALS.contains(&s) {
                // fourth close request will force the exit
                if cnt + 1 == 4 {
                    warn!("Received 4 close signals. Exiting now.");
                    println!("Received 4 close signals. Exiting now.");
                    process::exit(130);
                }
                (Some((sig, cnt + 1)), Msg::Control(ControlMsg::Close))
            } else {
                warn!("Received unknown signal {s}");
                (Some((sig, cnt)), Msg::None)
            }
        });
        ctrl.add_stream(stream);

        Ok(())
    }

    fn watch_config_task(
        sender: UnboundedSender<Msg>,
        watched: &Path,
    ) -> Result<INotifyWatcher> {
        let wat = watched.to_owned();
        let mut watcher = notify::recommended_watcher(move |res| {
            let v: notify::Event = match res {
                Ok(r) => r,
                Err(e) => {
                    error!("File watch failed: {e}");
                    return;
                }
            };

            if (v.kind.is_create() || v.kind.is_modify())
                && v.paths.contains(&wat)
            {
                if let Err(e) =
                    sender.unbounded_send(Msg::Config(ConfigMsg::Reload))
                {
                    error!("Failed to send message: {e}");
                }
            }
        })?;

        watcher.watch(watched, notify::RecursiveMode::NonRecursive)?;

        Ok(watcher)
    }

    fn server_task(
        listener: TcpListener,
        sender: UnboundedSender<Msg>,
    ) -> TcpListener {
        loop {
            let stream = listener.accept().unwrap();
            let mut msgr = Messenger::new(&stream.0);

            let rec = msgr.recieve();

            let rec = match rec {
                Ok(MsgMessage::Stop) => {
                    return listener;
                }
                Ok(MsgMessage::Ping) => {
                    if let Err(e) = msgr.send(MsgMessage::Success) {
                        error!("Failed to respond to ping: {e}");
                    }
                    continue;
                }
                Ok(m) => m,
                Err(e) => {
                    warn!("Failed to recieve message: {e}");
                    if let Err(e) =
                        msgr.send(MsgMessage::Error(messenger::Error::new(
                            messenger::ErrorKind::DeserializeFailed,
                            e.to_string(),
                        )))
                    {
                        warn!("Failed to send error message: {e}");
                    }
                    continue;
                }
            };

            let (response, msg) = Self::message_event(rec, &stream.0);
            if let Some(r) = response {
                if let Err(e) = msgr.send(r) {
                    warn!("Failed to send response {e}");
                }
            }

            if let Some(msg) = msg {
                if let Err(e) = sender.unbounded_send(msg) {
                    warn!("Failed to send message: {e}");
                }
            } else {
                continue;
            }
        }
    }
}
