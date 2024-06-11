use std::{
    collections::{hash_map::Entry, HashMap},
    net::TcpListener,
    thread::{self, JoinHandle},
};

use futures::channel::mpsc::UnboundedSender;
use log::error;

use crate::{
    core::{msg::Msg, Error, Result},
    library::{LibraryLoadResult, SongId},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TaskType {
    Server,
    LibraryLoad,
    LibrarySave,
}

#[derive(Debug)]
pub enum TaskMsg {
    Server(Result<TcpListener>),
    LibraryLoad(Result<Option<LibraryLoadResult>>),
    LibrarySave(Result<Vec<SongId>>),
}

#[derive(Debug)]
pub struct UniqueTasks {
    tasks: HashMap<TaskType, JoinHandle<TaskMsg>>,
    sender: UnboundedSender<Msg>,
}

impl UniqueTasks {
    pub fn new(sender: UnboundedSender<Msg>) -> Self {
        Self {
            tasks: HashMap::new(),
            sender,
        }
    }

    pub fn has_task(&self, id: TaskType) -> bool {
        self.tasks.contains_key(&id)
    }

    pub fn any<F>(&self, f: F) -> bool
    where
        F: Fn(TaskType) -> bool,
    {
        self.tasks.keys().copied().any(f)
    }

    pub fn check(&mut self) -> Vec<TaskMsg> {
        let keys: Vec<_> = self
            .tasks
            .iter()
            .filter_map(|(&k, v)| v.is_finished().then_some(k))
            .collect();
        keys.iter()
            .flat_map(|k| self.tasks.remove(k).map(|t| (k, t)))
            .map(|(k, t)| t.join().unwrap_or(k.panicked()))
            .collect()
    }

    pub fn add<F>(&mut self, typ: TaskType, f: F) -> Result<()>
    where
        F: FnOnce() -> TaskMsg + Send + 'static,
    {
        let ent = self.tasks.entry(typ);
        match ent {
            Entry::Occupied(_) => Err(Error::InvalidOperation(
                "The task of the type {typ} is already running",
            )),
            Entry::Vacant(e) => {
                let sender = self.sender.clone();
                e.insert(thread::spawn(move || {
                    let r = f();
                    if let Err(e) = sender.unbounded_send(Msg::default()) {
                        error!("Failed to send message: {e}");
                    }
                    r
                }));
                Ok(())
            }
        }
    }
}

impl TaskType {
    pub fn panicked(&self) -> TaskMsg {
        match self {
            Self::LibraryLoad => {
                TaskMsg::LibraryLoad(Err(Error::ThreadPanicked))
            }
            Self::LibrarySave => {
                TaskMsg::LibrarySave(Err(Error::ThreadPanicked))
            }
            Self::Server => TaskMsg::Server(Err(Error::ThreadPanicked)),
        }
    }
}
