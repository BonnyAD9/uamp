use std::{
    collections::{hash_map::Entry, HashMap},
    thread::{self, JoinHandle},
};

use futures::channel::mpsc::UnboundedSender;
use log::error;

use crate::core::{Error, Msg, Result, TaskMsg, TaskType};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Manager for tasks running on separate threads. Every task may run at most
/// once in any given time.
#[derive(Debug)]
pub struct UniqueTasks {
    tasks: HashMap<TaskType, JoinHandle<TaskMsg>>,
    sender: UnboundedSender<Msg>,
}

impl UniqueTasks {
    /// Create new task manager.
    ///
    /// - `sender`: Message is sent here when one of the tasks finishes.
    pub fn new(sender: UnboundedSender<Msg>) -> Self {
        Self {
            tasks: HashMap::new(),
            sender,
        }
    }

    /// Checks if there is any running task of the given type.
    pub fn has_task(&self, id: TaskType) -> bool {
        self.tasks.contains_key(&id)
    }

    /// Checks if there is any running task that matches the predicate.
    pub fn any<F>(&self, f: F) -> bool
    where
        F: Fn(TaskType) -> bool,
    {
        self.tasks.keys().copied().any(f)
    }

    /// Checks if any of the tasks have finished. This should be called when
    /// message about finished task is sent.
    pub fn check(&mut self) -> Vec<TaskMsg> {
        let keys: Vec<_> = self
            .tasks
            .iter()
            .filter_map(|(&k, v)| v.is_finished().then_some(k))
            .collect();
        keys.iter()
            .flat_map(|k| self.tasks.remove(k).map(|t| (k, t)))
            .map(|(k, t)| t.join().unwrap_or_else(|e| k.panicked(e)))
            .collect()
    }

    /// Start new task and add it to the manager.
    pub fn add<F>(&mut self, typ: TaskType, f: F) -> Result<()>
    where
        F: FnOnce() -> TaskMsg + Send + 'static,
    {
        let ent = self.tasks.entry(typ);
        match ent {
            Entry::Occupied(_) => Error::invalid_operation()
                .msg(format!("Failed to start task of type `{typ:?}`."))
                .reason("The task is already running.")
                .err(),
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
