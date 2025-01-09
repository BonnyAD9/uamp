use std::{any::Any, net::TcpListener};

use log::{error, warn};

use crate::env::AppCtrl;

use super::{
    library::{LibraryLoadResult, SongId},
    Error, Result, UampApp,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// List of task types that may run in Uamp.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TaskType {
    /// The TCP server task.
    Server,
    /// Library is loading new songs.
    LibraryLoad,
    /// Library is being saved to json.
    LibrarySave,
    /// Watches for signals.
    Signals,
}

/// Message sent from the task when it completes.
#[derive(Debug)]
pub enum TaskMsg {
    /// Server as finished, send its listener so that it may be reused when
    /// necesary
    Server(Result<TcpListener>),
    /// Library has finished loading. Sends its new library songs and what to
    /// do with them.
    LibraryLoad(Result<Option<LibraryLoadResult>>),
    /// Library has finished saving. Sends the temporary song ids that no
    /// longer have any references.
    LibrarySave(Result<Vec<SongId>>),
    /// Signals has ended.
    Signals(Result<()>),
}

impl TaskType {
    /// Creates [`TaskMsg`] of the same task type with information that the
    /// thread has panicked.
    pub fn panicked(&self, e: Box<dyn Any + Send + 'static>) -> TaskMsg {
        match self {
            Self::LibraryLoad => {
                TaskMsg::LibraryLoad(Err(Error::thread_panicked(Some(e))))
            }
            Self::LibrarySave => {
                TaskMsg::LibrarySave(Err(Error::thread_panicked(Some(e))))
            }
            Self::Server => {
                TaskMsg::Server(Err(Error::thread_panicked(Some(e))))
            }
            Self::Signals => {
                TaskMsg::Signals(Err(Error::thread_panicked(Some(e))))
            }
        }
    }

    /// Checks if uamp should wait for the task to finish before exiting.
    pub fn wait_before_exit(&self) -> bool {
        !matches!(self, Self::Server | Self::Signals)
    }
}

impl UampApp {
    /// Performes the correct action after a task has finished.
    pub fn task_end(&mut self, ctrl: &mut AppCtrl, task_res: TaskMsg) {
        if let Err(e) = self.task_end_err(ctrl, task_res) {
            error!("{}", e.log())
        }
    }

    pub fn task_end_err(
        &mut self,
        ctrl: &mut AppCtrl,
        task_res: TaskMsg,
    ) -> Result<()> {
        match task_res {
            TaskMsg::Server(Err(e)) => {
                e.prepend("Server unexpectedly ended.").err()
            }
            TaskMsg::Server(Ok(_)) => {
                if self.config.enable_server() || self.config.force_server {
                    Self::start_server(&self.config, ctrl, self.sender.clone())
                        .map_err(|e| e.prepend("Failed to restart server."))
                } else {
                    Ok(())
                }
            }
            TaskMsg::LibraryLoad(res) => self.finish_library_load(ctrl, res?),
            TaskMsg::LibrarySave(res) => self.finish_library_save_songs(res),
            TaskMsg::Signals(Err(e)) => {
                e.prepend("Signals task has unexpectedly ended.").err()
            }
            TaskMsg::Signals(Ok(_)) => {
                warn!("Signals task has unexpectedly ended, restarting.");
                Self::start_signal_thread(ctrl, self.sender.clone())
                    .map_err(|e| e.prepend("Failed to start signals thread."))
            }
        }
    }
}
