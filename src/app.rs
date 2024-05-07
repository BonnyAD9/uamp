use std::{
    cell::Cell,
    net::{TcpListener, TcpStream},
    process,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use futures::StreamExt;
use log::{error, warn};
use signal_hook_async_std::Signals;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{
    config::Config,
    core::{
        command::{ComMsg, Command},
        messenger::{self, Messenger, MsgMessage},
        msg::{ControlMsg, Msg},
        Error, Result,
    },
    library::Library,
    player::Player,
    tasks::{Task, TaskGen},
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
    pub sender: Arc<UnboundedSender<Msg>>,
    /// Reciever of the async messages
    pub reciever: Cell<Option<UnboundedReceiver<Msg>>>,

    /// The server listener
    pub listener: Cell<Option<TcpListener>>,

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
    pub fn save_all(&mut self) {
        match self
            .library
            .start_to_default_json(&self.config, self.sender.clone())
        {
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

    pub fn stop_server(&self, wait: Option<String>) {
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

    pub fn update(&mut self, message: Msg) -> Command {
        let com = match message {
            Msg::_PlaySong(index, songs) => {
                self.player.play_playlist(
                    &mut self.library,
                    songs,
                    Some(index),
                    true,
                );
                ComMsg::Msg(Msg::Tick)
            }
            Msg::Control(msg) => self.control_event(msg),
            Msg::Player(msg) => self.player_event(msg),
            Msg::Library(msg) => self.library_event(msg),
            Msg::Delegate(d) => d.update(self),
            Msg::Config(msg) => self.config_event(msg),
            Msg::Init => self.init(),
            Msg::Tick => ComMsg::none(),
            Msg::None => ComMsg::none(),
        };

        // The recursion that follows is all tail call recursion that will be
        // optimized so that there is no recursion

        let com = match com {
            ComMsg::Command(com) => com,
            ComMsg::Msg(msg) => return self.update(msg),
        };

        if self.pending_close && !self.library.any_process() {
            self.pending_close = false;
            return self.update(Msg::Control(ControlMsg::Close));
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
            self.library.any_process();
            self.save_all()
        }

        let up = self.library_lib_update();
        self.player_lib_update(up);

        com
    }
}

//===========================================================================//
//                                  Private                                  //
//===========================================================================//

impl UampApp {
    pub fn new(conf: Config) -> Result<Self> {
        let mut lib = Library::from_config(&conf);

        let (sender, reciever) = mpsc::unbounded_channel::<Msg>();
        let sender = Arc::new(sender);

        sender.send(Msg::Init)?;

        if conf.update_library_on_start() {
            lib.start_get_new_songs(&conf, sender.clone())?;
        }

        let mut player = Player::from_config(sender.clone(), &conf);
        player.load_config(&conf);
        player.remove_deleted(&lib);

        let listener = if conf.enable_server() || conf.force_server {
            Cell::new(Some(Self::start_server(&conf)?))
        } else {
            Cell::new(None)
        };

        Ok(UampApp {
            config: conf,
            library: lib,
            player,

            sender,
            reciever: Cell::new(Some(reciever)),

            listener,

            pending_close: false,

            last_save: Instant::now(),
            last_prev: Instant::now(),

            hard_pause_at: None,
        })
    }

    fn init(&mut self) -> ComMsg<Msg> {
        let mut res = ComMsg::none();

        if self.config.play_on_start() {
            res = ComMsg::Msg(Msg::Control(ControlMsg::PlayPause(Some(true))));
        }

        res
    }

    /// Starts the tcp server
    fn start_server(conf: &Config) -> Result<TcpListener> {
        Ok(TcpListener::bind(format!(
            "{}:{}",
            conf.server_address(),
            conf.port()
        ))?)
    }

    pub fn reciever_task(&self) -> Result<Box<dyn TaskGen<Msg>>> {
        if let Some(r) = self.reciever.take() {
            Ok(Box::new(Task::new(r, |mut r| async {
                let msg = r.recv().await.unwrap();
                (Some(r), msg)
            })))
        } else {
            Err(Error::InvalidOperation("reciever is already created"))
        }
    }

    pub fn signal_task(&self) -> Result<Box<dyn TaskGen<Msg>>> {
        let sig = Signals::new(signal_hook::consts::TERM_SIGNALS)?;
        Ok(Box::new(Task::new((sig, 0), |(mut sig, cnt)| async move {
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
        })))
    }

    pub fn run_server(&self) -> Result<()> {
        if let Some(listener) = self.listener.take() {
            let sender = self.sender.clone();
            thread::spawn(move || loop {
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
                        if let Err(e) = msgr
                            .send(messenger::msg::Message::Error(
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
                    if let Err(e) = sender.send(msg) {
                        warn!("Failed to send message: {e}");
                    }
                } else {
                    continue;
                }
            });
            Ok(())
        } else {
            Err(Error::InvalidOperation("Tcp listener is not available"))
        }
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
}
