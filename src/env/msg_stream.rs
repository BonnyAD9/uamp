use std::{fmt::Debug, mem};

use futures::{
    executor::block_on,
    future::{select_all, BoxFuture},
    Future,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Future that can produces message and may run again in its stream.
type MsgStreamFuture<Msg> =
    BoxFuture<'static, (Option<Box<dyn MsgStream<Msg>>>, Msg)>;

/// Asynchronous task that can generate messages.
pub trait MsgStream<Msg> {
    /// Gets the next future to run.
    fn next_future(self: Box<Self>) -> MsgStreamFuture<Msg>;
}

/// Asynchronous message generator.
#[derive(Debug)]
pub struct MsgGen<T, Msg, F, Fut>
where
    F: Fn(T) -> Fut + Send + 'static,
    Fut: Future<Output = (Option<T>, Msg)> + Send + 'static,
    Msg: 'static,
    T: 'static,
{
    data: T,
    fun: F,
}

/// Manages any number of asynchronous tasks.
pub struct Streams<Msg> {
    futures: Vec<MsgStreamFuture<Msg>>,
}

impl<T, Msg, F, Fut> MsgGen<T, Msg, F, Fut>
where
    F: Fn(T) -> Fut + Send,
    Fut: Future<Output = (Option<T>, Msg)> + Send + 'static,
{
    /// Creates new message generator.
    ///
    /// - `data`: state of the generator preserved between calls to `fun`.
    /// - `fun`: future that produces message.
    pub fn new(data: T, fun: F) -> Self {
        Self { data, fun }
    }
}

impl<T, Msg, F, Fut> MsgStream<Msg> for MsgGen<T, Msg, F, Fut>
where
    F: Fn(T) -> Fut + Send,
    Fut: Future<Output = (Option<T>, Msg)> + Send,
    T: Send,
{
    fn next_future(mut self: Box<Self>) -> MsgStreamFuture<Msg> {
        Box::pin(async move {
            let (data, m) = (self.fun)(self.data).await;
            if let Some(data) = data {
                *self = MsgGen {
                    data,
                    fun: self.fun,
                };
                let slf: Box<dyn MsgStream<_>> = self;
                (Some(slf), m)
            } else {
                (None, m)
            }
        })
    }
}

impl<Msg> Streams<Msg> {
    /// Creates new streams manager.
    pub fn new() -> Self {
        Self { futures: vec![] }
    }

    /// Block until any one of the asynchronous task to finish and return its
    /// result.
    pub fn wait_one(&mut self) -> Msg {
        let res = block_on(select_all(mem::take(&mut self.futures)));
        self.futures = res.2;
        if let Some(task) = res.0 .0 {
            self.futures.push(task.next_future());
        }
        res.0 .1
    }

    /// Add asynchronous task to the streams.
    pub fn add(&mut self, f: Box<dyn MsgStream<Msg>>) {
        self.futures.push(f.next_future());
    }
}

impl<Msg> Debug for Streams<Msg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Streams")
            .field("futures.len", &self.futures.len())
            .finish()
    }
}
