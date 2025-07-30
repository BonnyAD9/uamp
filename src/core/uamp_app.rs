use std::{
    env,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
    process::{self, Command},
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::{os::unix::process::CommandExt, rc::Rc};

use futures::executor::block_on;
use log::{error, info, trace, warn};
use mpris_server::LocalServer;
use notify::{INotifyWatcher, Watcher};
use tokio::signal::unix::SignalKind;
#[cfg(unix)]
use tokio::task::JoinHandle;

use crate::core::{
    AppCtrl, Error, Jobs, Result, RtAndle, RtHandle, State, config,
    library::Song, log_err, mpris::Mpris, msg::Msg,
};

use super::{
    ControlMsg, DataControlMsg,
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
    pub(super) rt: RtHandle,

    /// Running blocking jobs.
    pub(super) jobs: Jobs,

    /// Messages that can be processed only if there are no running processes
    pub(super) pending_close: bool,

    /// When this has value, it says when you can safely trigger hard pause.
    pub(super) hard_pause_at: Option<Instant>,

    /// When was last save
    pub(super) last_save: Instant,

    /// Las time that song was rewinded to the start with button.
    pub(super) last_prev: Instant,

    pub(super) restart_path: Option<PathBuf>,

    #[cfg(unix)]
    pub(super) mpris: Option<(Rc<LocalServer<Mpris>>, JoinHandle<()>)>,

    pub(super) state: State,

    file_watch: Option<INotifyWatcher>,
}

impl UampApp {
    /// Creates new uamp application instance.
    pub fn new(
        conf: Config,
        ctrl: &mut AppCtrl,
        rt: RtHandle,
    ) -> Result<Self> {
        let mut lib = Library::from_config(&conf);

        let mut player = Player::from_config(&mut lib, rt.andle(), &conf);
        player.load_config(&conf);
        player.retain(|s| !lib[s].is_deleted());

        // Signal stream is broken with receiver stream.
        //Self::signal_task(ctrl)?;
        Self::start_signals(ctrl)?;

        if conf.play_on_start() {
            rt.msg(ControlMsg::PlayPause(Some(true)).into());
        }

        let config_watch = if let Some(path) = &conf.config_path {
            match Self::watch_files(rt.andle(), path.as_path()) {
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

        let mut app = UampApp {
            config: conf,
            library: lib,
            player,

            rt,
            jobs: Jobs::default(),

            pending_close: false,

            last_save: Instant::now(),
            last_prev: Instant::now(),

            hard_pause_at: None,
            restart_path: None,

            #[cfg(unix)]
            mpris: None,

            state: State::default(),

            file_watch: config_watch,
        };

        if app.config.update_library_on_start() {
            app.start_get_new_songs(ctrl, Default::default())?;
        }

        if app.config.should_start_server() {
            app.start_server(ctrl)?;
        }

        let state = app.get_state();
        app.state = state;

        if app.config.system_player() {
            app.enable_system_player(ctrl);
        }

        Ok(app)
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
        self.update_many(ctrl, vec![message])
    }

    pub fn update_many(
        &mut self,
        ctrl: &mut AppCtrl,
        mut msgs: Vec<Msg>,
    ) -> Result<()> {
        trace!("{msgs:?}");
        #[cfg(debug_assertions)]
        dbg!(&msgs);

        msgs.reverse();
        let mut errs = vec![];

        while let Some(msg) = msgs.pop() {
            match self.msg_event(ctrl, msg) {
                Ok(r) => msgs.extend(r.into_iter().rev()),
                Err(e) => errs.push(e),
            }
        }

        if self.pending_close && !self.jobs.any_no_close() {
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

    pub(super) fn enable_system_player(&mut self, ctrl: &mut AppCtrl) {
        #[cfg(unix)]
        {
            if self.mpris.is_some() {
                return;
            }

            let e =
                LocalServer::new(config::APP_ID, Mpris::new(self.rt.clone()));
            let mpris: Option<Rc<_>> =
                log_err("Failed to start mpris player: ", block_on(e))
                    .map(|a| a.into());

            self.mpris = if let Some(s) = mpris {
                let task = s.run();
                Some((s, ctrl.spawn(task)))
            } else {
                None
            }
        }
    }

    pub(super) fn disable_system_player(&mut self) {
        #[cfg(unix)]
        {
            if let Some((_, h)) = self.mpris.take() {
                h.abort();
            }
        }
    }

    pub(super) fn get_state(&self) -> State {
        State {
            playback: self.player.playback_state(),
            cur_song: self
                .player
                .playlist()
                .current_idx()
                .map(|i| (self.player.playlist()[i], i)),
            volume: self.player.volume(),
            seeked: false,
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
            &mut self.jobs,
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

    /// Starts a thread for handling signals. This is only temorary workaround
    /// until a bug is fixed and `signal_task` will work properly.
    pub(super) fn start_signals(ctrl: &mut AppCtrl) -> Result<()> {
        #[cfg(unix)]
        let sigs = (
            tokio::signal::unix::signal(SignalKind::interrupt())?,
            tokio::signal::unix::signal(SignalKind::quit())?,
            tokio::signal::unix::signal(SignalKind::terminate())?,
        );
        #[cfg(windows)]
        let sigs = ();
        ctrl.unfold((0, sigs), |(mut n, mut sigs)| async move {
            #[cfg(windows)]
            tokio::signal::ctrl_c().await.unwrap();
            #[cfg(unix)]
            {
                tokio::select!(
                    _ = sigs.0.recv() => {}
                    _ = sigs.1.recv() => {}
                    _ = sigs.2.recv() => {}
                );
            }
            n += 1;
            if n == 4 {
                warn!("Received 4 close signals. Exiting now.");
                println!("Received 4 close signals. Exiting now.");
                process::exit(130);
            }
            Some((Msg::Control(ControlMsg::Close), (n, sigs)))
        });

        Ok(())
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl UampApp {
    fn routine(&mut self, errs: &mut Vec<Error>, ctrl: &mut AppCtrl) {
        let now = Instant::now();

        let up = self.library_routine();
        self.player_routine(now, up);
        errs.extend(self.config_routine(ctrl, now).err());
        errs.extend(self.restart(ctrl).err());
        #[cfg(unix)]
        self.mpris_routine(ctrl);
    }

    fn watch_files(rt: RtAndle, config_path: &Path) -> Result<INotifyWatcher> {
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
                } else if v.kind.is_remove() {
                    Some(Msg::fn_delegate(move |app, _| {
                        if let Some(ref mut watcher) = app.file_watch {
                            watcher.watch(
                                &path,
                                notify::RecursiveMode::NonRecursive,
                            )?;
                        }
                        Ok(vec![])
                    }))
                } else {
                    None
                };

                if let Some(msg) = msg {
                    rt.msg(msg);
                }
            }
        })?;

        watcher.watch(config_path, notify::RecursiveMode::NonRecursive)?;
        if let Some(p) = executable_path {
            watcher.watch(&p, notify::RecursiveMode::NonRecursive)?;
        }

        Ok(watcher)
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
