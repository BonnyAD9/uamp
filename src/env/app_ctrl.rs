use crate::core::{Msg, TaskMsg, TaskType};

use super::{Command, MsgStream, UniqueTasks};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Interface for performing actions in the environment wher UampApp runs.
#[derive(Debug)]
pub struct AppCtrl<'a> {
    commands: &'a mut Vec<Command>,
    tasks: &'a UniqueTasks,
}

impl<'a> AppCtrl<'a> {
    /// Creates new app control.
    pub fn new(
        commands: &'a mut Vec<Command>,
        tasks: &'a UniqueTasks,
    ) -> Self {
        Self { commands, tasks }
    }

    /// Add command to execute.
    pub fn add(&mut self, cmd: Command) {
        self.commands.push(cmd);
    }

    /// Request the app to exit.
    pub fn exit(&mut self) {
        self.add(Command::Exit)
    }

    /// Add asynchronous stream of messages to run.
    pub fn add_stream<S>(&mut self, s: S)
    where
        S: MsgStream<Option<Msg>> + 'static,
    {
        self.add(Command::AddStream(Box::new(s)))
    }

    /// Add task to run on a thread.
    pub fn add_task<T>(&mut self, id: TaskType, t: T)
    where
        T: FnOnce() -> TaskMsg + Send + 'static,
    {
        self.add(Command::AddTask(id, Box::new(t)))
    }

    /// Check if task of the given type is currently running.
    pub fn is_task_running(&self, id: TaskType) -> bool {
        self.tasks.has_task(id)
    }

    /// Check if any of the running tasks match the given predicate.
    pub fn any_task<P>(&self, f: P) -> bool
    where
        P: Fn(TaskType) -> bool,
    {
        self.tasks.any(&f)
            || self.commands.iter().any(|t| {
                if let Command::AddTask(t, _) = t {
                    f(*t)
                } else {
                    false
                }
            })
    }
}
