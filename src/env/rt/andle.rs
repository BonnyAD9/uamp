use futures::{
    Stream,
    stream::{self, BoxStream, StreamExt},
};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

use crate::{core::log_err, env::rt::Amsg};

#[derive(Debug)]
pub struct Andle<M: 'static, E: Send + 'static>(
    pub mpsc::UnboundedSender<Amsg<M, E>>,
);

impl<M, E: Send> Andle<M, E> {
    pub fn send(&self, m: Amsg<M, E>) {
        log_err("Failed to send message.", self.0.send(m));
    }

    pub fn msgs(&self, msgs: Vec<M>) {
        self.send(Amsg::Msg(msgs, None));
    }

    pub fn msg(&self, msg: M) {
        self.msgs(vec![msg]);
    }

    pub fn stream_rtx(&self, s: BoxStream<'static, Amsg<M, E>>) {
        self.send(Amsg::AddStream(s))
    }

    pub fn stream_rt(
        &self,
        s: impl Stream<Item = Amsg<M, E>> + Send + 'static,
    ) {
        self.stream_rtx(s.boxed());
    }

    pub fn streams(&self, s: impl Stream<Item = Vec<M>> + Send + 'static) {
        self.stream_rt(s.map(|a| Amsg::Msg(a, None)));
    }

    pub fn stream(&self, s: impl Stream<Item = M> + Send + 'static) {
        self.streams(s.map(|a| vec![a]));
    }

    pub fn unfold_rt<
        T: Send + 'static,
        F: Future<Output = Option<(Amsg<M, E>, T)>> + Send + 'static,
    >(
        &self,
        d: T,
        f: impl FnMut(T) -> F + Send + 'static,
    ) {
        self.stream_rt(stream::unfold(d, f))
    }

    pub fn unfold<
        T: Send + 'static,
        F: Future<Output = Option<(M, T)>> + Send + 'static,
    >(
        &self,
        d: T,
        f: impl FnMut(T) -> F + Send + 'static,
    ) {
        self.stream(stream::unfold(d, f))
    }

    pub fn task_rt(
        &self,
        f: impl Future<Output = Amsg<M, E>> + Send + 'static,
    ) {
        self.unfold_rt(Some(f), |f| async move {
            if let Some(f) = f {
                Some((f.await, None))
            } else {
                None
            }
        })
    }

    pub fn tasks(&self, f: impl Future<Output = Vec<M>> + Send + 'static) {
        self.task_rt(async move { Amsg::Msg(f.await, None) })
    }

    pub fn task(&self, f: impl Future<Output = M> + Send + 'static) {
        self.tasks(async move {
            let res = f.await;
            vec![res]
        })
    }

    pub fn spawn<T: Send + 'static>(
        &self,
        f: impl Future<Output = T> + Send + 'static,
    ) -> JoinHandle<T> {
        tokio::task::spawn(f)
    }

    pub async fn msgs_result(&self, m: Vec<M>) -> Result<(), E> {
        let (rsend, rrecv) = oneshot::channel();
        self.send(Amsg::Msg(m, Some(rsend)));
        log_err("Failed to receive message result.", rrecv.await)
            .unwrap_or(Ok(()))
    }

    pub async fn msg_result(&self, m: M) -> Result<(), E> {
        self.msgs_result(vec![m]).await
    }

    pub async fn tasks_result(
        &self,
        f: impl Future<Output = Vec<M>> + Send + 'static,
    ) -> Result<(), E> {
        let (rsend, rrecv) = oneshot::channel();
        self.task_rt(async move { Amsg::Msg(f.await, Some(rsend)) });
        log_err("Failed to receive message result.", rrecv.await)
            .unwrap_or(Ok(()))
    }

    pub async fn task_result(
        &self,
        f: impl Future<Output = M> + Send + 'static,
    ) -> Result<(), E> {
        self.tasks_result(async move {
            let res = f.await;
            vec![res]
        })
        .await
    }
}

impl<M, E: Send> Clone for Andle<M, E> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
