use tokio::sync::broadcast;

use crate::core::{
    RtAndle,
    server::{SubMsg, sub::SetAll},
};

pub struct SseService {
    rec: broadcast::Receiver<SubMsg>,
    init: Option<RtAndle>,
}

impl SseService {
    pub fn new(rec: broadcast::Receiver<SubMsg>, init: RtAndle) -> Self {
        Self {
            rec,
            init: Some(init),
        }
    }

    pub async fn next(&mut self) -> Option<String> {
        if let Some(init) = self.init.take() {
            Self::init(init).await
        } else {
            self.next_inner().await
        }
    }

    async fn init(rt: RtAndle) -> Option<String> {
        rt.request(|app, _| SubMsg::SetAll(SetAll::new(app)))
            .await
            .ok()?
            .event()
            .ok()
    }

    async fn next_inner(&mut self) -> Option<String> {
        self.rec.recv().await.ok()?.event().ok()
    }
}
