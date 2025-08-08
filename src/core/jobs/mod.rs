mod job;
mod job_msg;

#[cfg(unix)]
use std::rc::Rc;

use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

#[cfg(unix)]
use crate::core::mpris::Mpris;
use crate::core::server::SubMsg;

pub use self::{job::*, job_msg::*};

#[derive(Debug, Default)]
pub struct Jobs {
    unique: Job,
    pub server: Option<(broadcast::Sender<SubMsg>, CancellationToken)>,
    #[cfg(unix)]
    pub system_player: Option<Rc<mpris_server::Server<Mpris>>>,
}

impl Jobs {
    pub fn any_no_close(&self) -> bool {
        self.unique.intersects(Job::NO_CLOSE)
    }

    pub fn is_running(&self, job: Job) -> bool {
        self.unique.contains(job)
    }

    pub fn run(&mut self, job: Job) {
        self.unique.set(job, true);
    }

    pub fn finish(&mut self, job: Job) {
        self.unique.set(job, false);
    }
}
