use tokio::sync::broadcast;

use crate::core::{
    RtAndle, log_err,
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
        log_err(
            "Failed to create sse message: ",
            rt.request(|app, _| SubMsg::SetAll(SetAll::new(app).into()))
                .await
                .ok()?
                .event(),
        )
    }

    async fn next_inner(&mut self) -> Option<String> {
        self.rec.recv().await.ok()?.event().ok()
    }
}
