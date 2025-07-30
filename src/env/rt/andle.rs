use tokio::sync::mpsc;

use crate::core::log_err;

pub struct Andle<M>(pub mpsc::UnboundedSender<M>);

impl<M> Andle<M> {
    pub fn send(&self, m: M) {
        log_err("Failed to send message.", self.0.send(m));
    }
}

impl<M> Clone for Andle<M> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
