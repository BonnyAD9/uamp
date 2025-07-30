use std::task::Poll;

use futures::{StreamExt, stream::LocalBoxStream};

pub struct StreamFuture<Msg>(Option<LocalBoxStream<'static, Msg>>);

impl<Msg> Future for StreamFuture<Msg> {
    type Output = (Option<Msg>, LocalBoxStream<'static, Msg>);

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let res = self.0.as_mut().unwrap().poll_next_unpin(cx);
        match res {
            Poll::Pending => Poll::Pending,
            Poll::Ready(r) => Poll::Ready((r, self.0.take().unwrap())),
        }
    }
}

impl<Msg> StreamFuture<Msg> {
    pub fn new(stream: LocalBoxStream<'static, Msg>) -> Self {
        Self(Some(stream))
    }
}
