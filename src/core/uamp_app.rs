use std::{
    env,
    fs::{self, DirEntry},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{self, Command},
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use futures::{StreamExt, channel::mpsc::UnboundedSender};
use log::{error, info, trace, warn};
use notify::{INotifyWatcher, Watcher};
use signal_hook_async_std::Signals;

use crate::{
    core::{
        Error, Result,
        library::Song,
        messenger::{self, Messenger, MsgMessage},
        msg::Msg,
    },
    env::{AppCtrl, MsgGen, State},
};

use super::{
    ControlMsg, DataControlMsg, TaskMsg, TaskType,
    config::{Config, ConfigMsg, default_log_dir},
    library::{Library, SongId},
    player::Player,
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

    pub(super) restart_path: Option<PathBuf>,

    _file_watch: Option<INotifyWatcher>,
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
        player.retain(|s| !lib[s].is_deleted());

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
            match Self::watch_files(sender.clone(), path.as_path()) {
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
            restart_path: None,

            _file_watch: config_watch,
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

    /// Runs when uamp is about to exit.
    pub fn on_exit(&mut self) {
        if let Err(e) = delete_old_logs(self.config.delete_logs_after().0) {
            error!("Failed to delete old logs: {}", e.log());
        }
    }

    pub fn get_state(&self) -> State {
        State {
            playback: self.player.playback_state(),
            cur_song: self
                .player
                .playlist()
                .current_idx()
                .map(|i| (self.player.playlist()[i], i)),
            volume: self.player.volume(),
        }
    }

    pub fn get_song(&self, id: SongId) -> &Song {
        &self.library[id]
    }

    /// Old song ids were replaced with new valid song ids.
    pub(super) fn id_replace(&mut self, n: impl Fn(SongId, &Library) -> bool) {
        self.player_id_replace(n);
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

    pub(super) fn save_all_block(&mut self, closing: bool) -> Result<()> {
        let mut res = vec![];
        match self
            .library
            .save_to_default_json(&self.config, &mut self.player)
        {
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
        self.player_routine(ctrl, now, up);
        errs.extend(self.config_routine(ctrl, now).err());
        errs.extend(self.restart(ctrl).err());
    }

    fn watch_files(
        sender: UnboundedSender<Msg>,
        config_path: &Path,
    ) -> Result<INotifyWatcher> {
        let cfg = config_path.to_owned();

        let executable_path = match env::current_exe() {
            Err(e) => {
                error!("Failed to retrieve current executable path: {e}");
                None
            }
            Ok(p) => Some(p),
        };

        let exe = executable_path.clone().unwrap_or_default();

        let mut watcher = notify::recommended_watcher(move |res| {
            let v: notify::Event = match res {
                Ok(r) => r,
                Err(e) => {
                    error!("File watch failed: {e}");
                    return;
                }
            };

            for path in v.paths {
                let msg = if path == cfg
                    && (v.kind.is_create() || v.kind.is_modify())
                {
                    Some(Msg::Config(ConfigMsg::Reload))
                } else if path == exe
                    && (v.kind.is_remove()
                        || v.kind.is_create()
                        || v.kind.is_modify())
                {
                    Some(DataControlMsg::Restart(Some(path)).into())
                } else {
                    None
                };

                if let Some(msg) = msg {
                    if let Err(e) = sender.unbounded_send(msg) {
                        error!("Failed to send message: {e}");
                    }
                }
            }
        })?;

        watcher.watch(config_path, notify::RecursiveMode::NonRecursive)?;
        if let Some(p) = executable_path {
            watcher.watch(&p, notify::RecursiveMode::NonRecursive)?;
        }

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

    fn restart(&mut self, ctrl: &mut AppCtrl) -> Result<()> {
        let Some(exe) = &self.restart_path else {
            return Ok(());
        };

        if !exe.exists() {
            warn!(
                "Cannot restart now. The new uamp binary doesn't exist. \
                Waiting for the new binary to exist."
            );
            return Ok(());
        }

        info!("Restarting uamp.");

        let exe = exe.clone();

        self.save_all_block(true)?;

        let mut cmd = Command::new(exe);
        cmd.arg("run");
        self.construct_state(|arg| {
            cmd.arg(arg);
        });

        #[cfg(unix)]
        let res = Err(cmd.exec());
        #[cfg(not(unix))]
        let res = cmd.spawn();

        res.map_err(|e| {
            Error::from(e).msg("Failed to start new instance of uamp.")
        })?;

        info!("New uamp started. Exec not supported. Quitting.");

        ctrl.exit();
        Ok(())
    }

    fn construct_state(&self, mut arg: impl FnMut(&str)) {
        if self.config.config_path.is_none() {
            arg("-p");
            arg(&self.config.port().to_string());
            arg("-a");
            arg(self.config.server_address());
        }

        let play = self.player.is_playing();
        arg(&ControlMsg::PlayPause(Some(play)).to_string());

        if let Some(pos) = self.player.timestamp() {
            arg(&ControlMsg::SeekTo(pos.current).to_string());
        }
    }
}

/// Deletes old logs.
fn delete_old_logs(timeout: Duration) -> Result<()> {
    let dir = fs::read_dir(default_log_dir())?;

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
