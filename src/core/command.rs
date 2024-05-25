use std::fmt::Debug;

use crate::sync::{
    msg_stream::MsgStream,
    tasks::{TaskMsg, TaskType, UniqueTasks},
};

use super::msg::Msg;

#[derive(Debug)]
pub struct AppCtrl<'a> {
    commands: &'a mut Vec<Command>,
    tasks: &'a UniqueTasks,
}

pub enum Command {
    Exit,
    AddStream(Box<dyn MsgStream<Msg>>),
    AddTask(TaskType, Box<dyn FnOnce() -> TaskMsg + Send + 'static>),
}

impl<'a> AppCtrl<'a> {
    pub fn new(
        commands: &'a mut Vec<Command>,
        tasks: &'a UniqueTasks,
    ) -> Self {
        Self { commands, tasks }
    }

    pub fn add(&mut self, cmd: Command) {
        self.commands.push(cmd);
    }

    pub fn exit(&mut self) {
        self.add(Command::Exit)
    }

    pub fn add_stream<S>(&mut self, s: S)
    where
        S: MsgStream<Msg> + 'static,
    {
        self.add(Command::AddStream(Box::new(s)))
    }

    pub fn add_task<T>(&mut self, id: TaskType, t: T)
    where
        T: FnOnce() -> TaskMsg + Send + 'static,
    {
        self.add(Command::AddTask(id, Box::new(t)))
    }

    pub fn is_task_running(&self, id: TaskType) -> bool {
        self.tasks.has_task(id)
    }

    pub fn any_task<P>(&self, f: P) -> bool
    where
        P: Fn(TaskType) -> bool,
    {
        self.tasks.any(f)
    }
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exit => write!(f, "Exit"),
            Self::AddStream(_) => f.debug_tuple("AddStream").finish(),
            Self::AddTask(t, _) => f.debug_tuple("AddTask").field(t).finish(),
        }
    }
}
