use tokio::sync::broadcast;

use crate::core::{
    Error, Result, RtAndle,
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

    pub async fn next(&mut self) -> String {
        if let Some(init) = self.init.take() {
            Self::init(init).await
        } else {
            self.next_inner().await
        }
    }

    async fn init(rt: RtAndle) -> String {
        guard_error(Self::init_inner(rt).await)
    }

    async fn init_inner(rt: RtAndle) -> Result<String> {
        rt.request(|app, _| SubMsg::SetAll(SetAll::new(app)))
            .await?
            .event()
    }

    async fn next_inner(&mut self) -> String {
        guard_error(self.next_res().await)
    }

    async fn next_res(&mut self) -> Result<String> {
        self.rec
            .recv()
            .await
            .map_err(|_| {
                Error::Unexpected(
                    "Event stream has been closed by uamp.".into(),
                )
            })?
            .event()
    }
}

fn guard_error(res: Result<String>) -> String {
    match res {
        Ok(m) => m,
        Err(e) => format!(
            "event: error\ndata: {}\n\n",
            e.log().to_string().replace('\n', "\ndata: ")
        ),
    }
}
