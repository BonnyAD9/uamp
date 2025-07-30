mod job;
mod job_msg;

use tokio_util::sync::CancellationToken;

pub use self::{job::*, job_msg::*};

#[derive(Debug, Default)]
pub struct Jobs {
    unique: Job,
    pub server: Option<CancellationToken>,
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
