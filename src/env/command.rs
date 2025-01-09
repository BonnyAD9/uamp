use std::fmt::Debug;

use crate::core::{Msg, TaskMsg, TaskType};

use super::MsgStream;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Command to the UampApp environment.
pub enum Command {
    /// Request exit.
    Exit,
    /// Add stream to run asynchronously.
    _AddStream(Box<dyn MsgStream<Msg>>),
    /// Add task to run in parallel on a thread.
    AddTask(TaskType, Box<dyn FnOnce() -> TaskMsg + Send + 'static>),
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exit => write!(f, "Exit"),
            Self::_AddStream(_) => f.debug_tuple("AddStream").finish(),
            Self::AddTask(t, _) => f.debug_tuple("AddTask").field(t).finish(),
        }
    }
}
