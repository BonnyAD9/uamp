use std::{
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};

use itertools::Either;
use tokio::sync::{broadcast};
use tokio_util::sync::CancellationToken;

use crate::core::{config::Config, server::SubMsg};

#[derive(Debug, Clone)]
pub struct ServerData {
    pub sender:
        Either<broadcast::Sender<SubMsg>, broadcast::WeakSender<SubMsg>>,
    pub cancel: CancellationToken,
    pub cache: Arc<RwLock<PathBuf>>,
    pub client: Arc<RwLock<PathBuf>>,
}

const MAX_BROADCAST_CAPACITY: usize = 16;

impl ServerData {
    pub fn new(conf: &Config) -> Self {
        let (sender, _) = broadcast::channel(MAX_BROADCAST_CAPACITY);
        Self {
            sender: Either::Left(sender),
            cancel: CancellationToken::new(),
            cache: Arc::new(conf.cache_path().clone().into()),
            client: Arc::new(conf.http_client().clone().into()),
        }
    }

    pub fn weak_clone(&self) -> Self {
        let mut res = self.clone();
        res.downgrade();
        res
    }

    pub fn downgrade(&mut self) {
        self.sender = match self.sender {
            Either::Left(ref l) => Either::Right(l.downgrade()),
            ref v => v.clone(),
        }
    }

    pub fn strong_send(
        &self,
        msg: SubMsg,
    ) -> Result<(), broadcast::error::SendError<SubMsg>> {
        self.sender
            .as_ref()
            .left()
            .map(|a| a.send(msg))
            .map_or_else(|| Ok(()), |r| r.map(|_| ()))
    }

    pub fn make_reciever(&self) -> Option<broadcast::Receiver<SubMsg>> {
        self.sender.as_ref().either(
            |a| Some(a.subscribe()),
            |a| a.upgrade().map(|a| a.subscribe()),
        )
    }
}
