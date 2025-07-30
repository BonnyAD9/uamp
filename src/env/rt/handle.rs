use futures::{
    Stream, StreamExt,
    stream::{self, LocalBoxStream},
};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

use crate::{
    core::log_err,
    env::rt::{Andle, Msg},
};

pub struct Handle<M: 'static, E: 'static> {
    pub local: mpsc::UnboundedSender<Msg<M, E>>,
    pub thread: mpsc::UnboundedSender<M>,
}

impl<M, E> Handle<M, E> {
    pub fn send(&self, m: Msg<M, E>) {
        log_err("Failed to send message.", self.local.send(m));
    }

    pub fn msgs(&self, msgs: Vec<M>) {
        self.send(Msg::Msg(msgs, None));
    }

    pub fn msg(&self, msg: M) {
        self.msgs(vec![msg]);
    }

    pub fn stream_rtx(&self, s: LocalBoxStream<'static, Msg<M, E>>) {
        self.send(Msg::AddStream(s))
    }

    pub fn stream_rt(&self, s: impl Stream<Item = Msg<M, E>> + 'static) {
        self.stream_rtx(s.boxed_local());
    }

    pub fn streams(&self, s: impl Stream<Item = Vec<M>> + 'static) {
        self.stream_rt(s.map(|a| Msg::Msg(a, None)));
    }

    pub fn stream(&self, s: impl Stream<Item = M> + 'static) {
        self.streams(s.map(|a| vec![a]));
    }

    pub fn unfold_rt<
        T: 'static,
        F: Future<Output = Option<(Msg<M, E>, T)>> + 'static,
    >(
        &self,
        d: T,
        f: impl FnMut(T) -> F + 'static,
    ) {
        self.stream_rt(stream::unfold(d, f))
    }

    pub fn unfold<T: 'static, F: Future<Output = Option<(M, T)>> + 'static>(
        &self,
        d: T,
        f: impl FnMut(T) -> F + 'static,
    ) {
        self.stream(stream::unfold(d, f))
    }

    pub fn task_rt(&self, f: impl Future<Output = Msg<M, E>> + 'static) {
        self.unfold_rt(Some(f), |f| async move {
            if let Some(f) = f {
                Some((f.await, None))
            } else {
                None
            }
        })
    }

    pub fn tasks(&self, f: impl Future<Output = Vec<M>> + 'static) {
        self.task_rt(async move { Msg::Msg(f.await, None) })
    }

    pub fn task(&self, f: impl Future<Output = M> + 'static) {
        self.tasks(async move { vec![f.await] })
    }

    pub fn spawn<T: 'static>(
        &self,
        f: impl Future<Output = T> + 'static,
    ) -> JoinHandle<T> {
        tokio::task::spawn_local(f)
    }

    pub async fn msgs_result(&self, m: Vec<M>) -> Result<(), E> {
        let (rsend, rrecv) = oneshot::channel();
        self.send(Msg::Msg(m, Some(rsend)));
        log_err("Failed to receive message result.", rrecv.await)
            .unwrap_or(Ok(()))
    }

    pub async fn msg_result(&self, m: M) -> Result<(), E> {
        self.msgs_result(vec![m]).await
    }

    pub async fn tasks_result(
        &self,
        f: impl Future<Output = Vec<M>> + 'static,
    ) -> Result<(), E> {
        let (rsend, rrecv) = oneshot::channel();
        self.task_rt(async move { Msg::Msg(f.await, Some(rsend)) });
        log_err("Failed to receive message result.", rrecv.await)
            .unwrap_or(Ok(()))
    }

    pub async fn task_result(
        &self,
        f: impl Future<Output = M> + 'static,
    ) -> Result<(), E> {
        self.tasks_result(async move { vec![f.await] }).await
    }

    pub fn andle(&self) -> Andle<M> {
        Andle(self.thread.clone())
    }
}

impl<M, E> Clone for Handle<M, E> {
    fn clone(&self) -> Self {
        Self {
            local: self.local.clone(),
            thread: self.thread.clone(),
        }
    }
}
