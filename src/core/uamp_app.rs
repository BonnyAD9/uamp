use std::{
    fs::{self, DirEntry},
    net::TcpListener,
    path::Path,
    process,
    time::{Duration, Instant},
};

use futures::{channel::mpsc::UnboundedSender, StreamExt};
use log::{error, trace, warn};
use notify::{INotifyWatcher, Watcher};
use signal_hook_async_std::Signals;

use crate::{
    core::{
        messenger::{self, Messenger, MsgMessage},
        msg::Msg,
        Error, Result,
    },
    env::{AppCtrl, MsgGen},
};

use super::{
    config::{default_config_dir, Config, ConfigMsg},
    library::Library,
    player::Player,
    ControlMsg, TaskMsg, TaskType,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// The uamp app state
pub struct UampApp {
    /// The configuration
    pub(super) config: Config,
    /// The song library
    pub(super) library: Library,
    /// The song player
    pub(super) player: Player,

    /// Sender for async messages to be synchronized with the main message
    /// handler
    pub(super) sender: UnboundedSender<Msg>,

    /// Messages that can be processed only if there are no running processes
    pub(super) pending_close: bool,

    /// When this has value, it says when you can safely trigger hard pause.
    pub(super) hard_pause_at: Option<Instant>,

    /// When was last save
    pub(super) last_save: Instant,

    /// Las time that song was rewinded to the start with button.
    pub(super) last_prev: Instant,

    _config_watch: Option<INotifyWatcher>,
}

impl UampApp {
    /// Creates new uamp application instance.
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

        // Signal stream is broken with receiver stream.
        //Self::signal_task(ctrl)?;
        Self::start_signal_thread(ctrl, sender.clone())?;

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
                    error!("Failed to watch config file: {}", e.log());
                    None
                }
            }
        } else {
            None
        };

        if let Err(e) = delete_old_logs(conf.delete_logs_after().0) {
            error!("Failed to delete old logs: {}", e.log());
        }

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

    /// Handles the message sent to uamp.
    pub fn update(&mut self, ctrl: &mut AppCtrl, message: Msg) {
        if let Err(e) = self.update_err(ctrl, message) {
            error!("{}", e.log());
        }
    }

    /// Handles the message sent to uamp, propagates errors.
    pub fn update_err(
        &mut self,
        ctrl: &mut AppCtrl,
        message: Msg,
    ) -> Result<()> {
        trace!("{message:?}");
        #[cfg(debug_assertions)]
        dbg!(&message);

        let mut msgs = self.msg_event(ctrl, message)?;
        if msgs.len() == 1 {
            return self.update_err(ctrl, msgs.pop().unwrap());
        }

        msgs.reverse();
        let mut errs = vec![];

        while let Some(msg) = msgs.pop() {
            match self.msg_event(ctrl, msg) {
                Ok(r) => msgs.extend(r.into_iter().rev()),
                Err(e) => errs.push(e),
            }
        }

        if self.pending_close && !ctrl.any_task(|t| t.wait_before_exit()) {
            self.pending_close = false;
            return self.update_err(ctrl, Msg::Control(ControlMsg::Close));
        }

        self.routine(&mut errs, ctrl);

        Error::multiple(errs)
    }

    pub fn on_exit(&mut self) {
        if let Err(e) = delete_old_logs(self.config.delete_logs_after().0) {
            error!("Failed to delete old logs: {}", e.log());
        }
    }

    /// Saves all the data that is saved by uamp.
    pub(super) fn save_all(
        &mut self,
        closing: bool,
        ctrl: &mut AppCtrl,
    ) -> Result<()> {
        let mut res = vec![];
        match self.library.start_to_default_json(
            &self.config,
            ctrl,
            &mut self.player,
        ) {
            Err(Error::InvalidOperation(_)) => {}
            Err(e) => res.push(e.prepend("Failed to start library save.")),
            _ => {}
        }
        if let Err(e) = self.player.save_to_default_json(closing, &self.config)
        {
            res.push(e.prepend("Failed to save play state."));
        }
        if let Err(e) = self.config.to_default_json() {
            res.push(e.prepend("Failed to save config."));
        }

        self.last_save = Instant::now();
        Error::multiple(res)
    }

    /// Starts the tcp server.
    pub(super) fn start_server(
        conf: &Config,
        ctrl: &mut AppCtrl,
        sender: UnboundedSender<Msg>,
    ) -> Result<()> {
        if ctrl.is_task_running(TaskType::Server) {
            return Error::invalid_operation()
                .msg("Failed to start server.")
                .reason("Server is already running.")
                .err();
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

    /// Starts a thread for handling signals. This is only temorary workaround
    /// until a bug is fixed and `signal_task` will work properly.
    pub(super) fn start_signal_thread(
        ctrl: &mut AppCtrl,
        sender: UnboundedSender<Msg>,
    ) -> Result<()> {
        if ctrl.is_task_running(TaskType::Signals) {
            return Error::invalid_operation()
                .msg("Failed to start signals thread.")
                .reason("Signals thread is already running.")
                .err();
        }

        let signals = signal_hook::iterator::Signals::new(
            signal_hook::consts::TERM_SIGNALS,
        )?;

        let task = move || {
            TaskMsg::Signals({
                Self::signal_thread(sender, signals);
                Ok(())
            })
        };
        ctrl.add_task(TaskType::Signals, task);

        Ok(())
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl UampApp {
    fn _signal_task(ctrl: &mut AppCtrl) -> Result<()> {
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
        ctrl._add_stream(stream);

        Ok(())
    }

    fn routine(&mut self, errs: &mut Vec<Error>, ctrl: &mut AppCtrl) {
        let now = Instant::now();

        let up = self.library_routine();
        self.player_routine(now, up);
        errs.extend(self.config_routine(ctrl, now).err());
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
                        error!("Failed to respond to ping: {}", e.log());
                    }
                    continue;
                }
                Ok(m) => m,
                Err(mut e) => {
                    e = e.log();
                    warn!("Failed to recieve message: {e}");
                    if let Err(e) =
                        msgr.send(MsgMessage::Error(messenger::Error::new(
                            messenger::ErrorKind::DeserializeFailed,
                            e.clone_universal(),
                        )))
                    {
                        warn!("Failed to send error message: {}", e.log());
                    }
                    continue;
                }
            };

            let (response, msg) = Self::message_event(rec, &stream.0);
            if let Some(r) = response {
                if let Err(e) = msgr.send(r) {
                    warn!("Failed to send response {}", e.log());
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

    fn signal_thread(
        sender: UnboundedSender<Msg>,
        mut signals: signal_hook::iterator::Signals,
    ) {
        let mut cnt = 0;

        for sig in &mut signals {
            if signal_hook::consts::TERM_SIGNALS.contains(&sig) {
                cnt += 1;
                // fourth close request will force the exit
                if cnt == 4 {
                    warn!("Received 4 close signals. Exiting now.");
                    println!("Received 4 close signals. Exiting now.");
                    process::exit(130);
                }
                if let Err(e) =
                    sender.unbounded_send(Msg::Control(ControlMsg::Close))
                {
                    error!("Failed to send close message: {e}");
                }
            } else {
                warn!("Received unknown signal {sig}");
            }
        }
    }
}

/// Deletes old logs.
fn delete_old_logs(timeout: Duration) -> Result<()> {
    let dir = fs::read_dir(default_config_dir().join("log"))?;

    for d in dir {
        let d = d?;
        if let Err(e) = delete_old_log(&d, timeout) {
            error!("Failed to delete log file {:?}: {}", d.path(), e.log());
        }
    }

    Ok(())
}

fn delete_old_log(file: &DirEntry, timeout: Duration) -> Result<()> {
    let mt = file.metadata()?.modified()?;
    if mt.elapsed()? > timeout {
        fs::remove_file(file.path())?;
    }
    Ok(())
}
