use std::fmt::Debug;

use futures::stream::LocalBoxStream;

use crate::env::rt;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Command to the UampApp environment.
pub enum Command<M, E> {
    /// Request exit.
    Exit,
    AddStrem(LocalBoxStream<'static, rt::Msg<M, E>>),
}

impl<M, E> Debug for Command<M, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exit => write!(f, "Exit"),
            Self::AddStrem(_) => f.debug_tuple("AddStrem").finish(),
        }
    }
}
