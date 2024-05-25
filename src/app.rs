use std::{
    net::{TcpListener, TcpStream},
    process, thread,
    time::{Duration, Instant},
};

use futures::{channel::mpsc::UnboundedSender, StreamExt};
use log::{error, warn};
use signal_hook_async_std::Signals;

use crate::{
    config::Config,
    core::{
        command::AppCtrl,
        messenger::{self, Messenger, MsgMessage},
        msg::{ControlMsg, Msg},
        Error, Result,
    },
    library::Library,
    player::Player,
    sync::{
        msg_stream::MsgGen,
        tasks::{TaskMsg, TaskType},
    },
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
}

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

impl UampApp {
    /// Saves all the data that is saved by uamp
    pub fn save_all(&mut self, ctrl: &mut AppCtrl) {
        match self.library.start_to_default_json(&self.config, ctrl) {
            Err(Error::InvalidOperation(_)) => {}
            Err(e) => error!("Failed to start library save: {e}"),
            _ => {}
        }
        if let Err(e) = self.player.to_default_json(&self.config) {
            error!("Failed to save play state: {e}");
        }
        if let Err(e) = self.config.to_default_json() {
            error!("Failed to save config: {e}");
        }

        self.last_save = Instant::now();
    }

    pub fn _stop_server(&self, wait: Option<String>) {
        let adr =
            format!("{}:{}", self.config.server_address(), self.config.port());
        thread::spawn(move || {
            let err = (|| -> Result<()> {
                if let Some(adr) = wait {
                    let s = TcpStream::connect(adr)?;
                    if let Err(e) = s.set_nodelay(true) {
                        warn!("Failed to set no delay when waiting to stop server: {e}");
                    }
                    s.set_write_timeout(Some(Duration::from_secs(5)))?;
                    let mut msg = Messenger::try_new(&s)?;
                    msg.send(MsgMessage::WaitExit(Duration::from_secs(5)))?;
                    match msg.recieve()? {
                        MsgMessage::Success => {}
                        m => {
                            warn!("Unexpected response when pinging new server before stopping the old one: {m:?}");
                        }
                    }
                }

                let s = TcpStream::connect(adr)?;
                if let Err(e) = s.set_nodelay(true) {
                    warn!("Failed to set no delay when stopping server: {e}");
                }
                s.set_write_timeout(Some(Duration::from_secs(5)))?;
                let mut msg = Messenger::try_new(&s)?;
                msg.send(MsgMessage::WaitExit(Duration::from_secs(0)))?;
                Ok(())
            })();

            if let Err(e) = err {
                error!("Failed to stop server: {e}");
            }
        });
    }

    pub fn task_end(&mut self, ctrl: &mut AppCtrl, task_res: TaskMsg) {
        match task_res {
            TaskMsg::Server(Err(e)) => {
                error!("Server unexpectedly ended: {e}");
            }
            TaskMsg::Server(Ok(_)) => {}
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
            Msg::_PlaySong(index, songs) => {
                self.player.play_playlist(
                    &mut self.library,
                    songs,
                    Some(index),
                    true,
                );
                Some(Msg::Tick)
            }
            Msg::Control(msg) => self.control_event(ctrl, msg),
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
            self.save_all(ctrl);
        }

        let up = self.library_lib_update();
        self.player_lib_update(up);
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl UampApp {
    pub fn new(
        conf: Config,
        ctrl: &mut AppCtrl,
        sender: UnboundedSender<Msg>,
    ) -> Result<Self> {
        let mut lib = Library::from_config(&conf);

        if conf.update_library_on_start() {
            println!("Update lib");
            lib.start_get_new_songs(&conf, ctrl, Default::default())?;
        }

        let mut player = Player::from_config(sender.clone(), &conf);
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

        Ok(UampApp {
            config: conf,
            library: lib,
            player,

            sender,

            pending_close: false,

            last_save: Instant::now(),
            last_prev: Instant::now(),

            hard_pause_at: None,
        })
    }

    /// Starts the tcp server
    fn start_server(
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

    pub fn signal_task(ctrl: &mut AppCtrl) -> Result<()> {
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

    /*fn clock_subscription(&self, tick: Duration) -> Subscription {
        iced::subscription::unfold(
            format!("{} tick ({})", app_id(), duration_to_string(tick, false)),
            Instant::now(),
            move |i| async move {
                let now = Instant::now();
                let dif = now - i;
                if dif < tick {
                    thread::sleep(tick - dif);
                }
                (Msg::Gui(GuiMessage::Tick), i + tick)
            },
        )
    }*/

    fn server_task(
        listener: TcpListener,
        sender: UnboundedSender<Msg>,
    ) -> TcpListener {
        loop {
            let stream = listener.accept().unwrap();
            let mut msgr = Messenger::try_new(&stream.0).unwrap();

            let rec = msgr.recieve();

            let rec = match rec {
                Ok(MsgMessage::WaitExit(d)) => {
                    thread::sleep(d);
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
                    if let Err(e) = msgr.send(messenger::msg::Message::Error(
                        messenger::msg::Error::new(
                            messenger::msg::ErrorType::DeserializeFailed,
                            e.to_string(),
                        ),
                    )) {
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
