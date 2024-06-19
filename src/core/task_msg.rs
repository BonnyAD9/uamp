use std::net::TcpListener;

use super::{
    library::{LibraryLoadResult, SongId},
    Error, Result,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// List of task types that may run in Uamp.
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
