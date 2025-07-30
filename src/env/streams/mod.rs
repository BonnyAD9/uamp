use ::futures::stream::LocalBoxStream;
use log::error;
use tokio::task::JoinSet;

mod stream_future;

pub use self::stream_future::*;

#[derive(Debug)]
pub struct Streams<Msg: 'static>(
    JoinSet<<StreamFuture<Msg> as Future>::Output>,
);

impl<Msg> Streams<Msg> {
    pub fn new() -> Self {
        Self(JoinSet::new())
    }

    pub fn add_stream(&mut self, s: LocalBoxStream<'static, Msg>) {
        self.0.spawn_local(StreamFuture::new(s));
    }

    pub async fn next(&mut self) -> Option<Msg> {
        loop {
            match self.0.join_next().await? {
                Ok((Some(msg), stream)) => {
                    self.add_stream(stream);
                    return Some(msg);
                }
                Err(e) => {
                    if !e.is_cancelled() {
                        error!("Streams task failed: {e}");
                    }
                }
                _ => {}
            }
        }
    }
}
