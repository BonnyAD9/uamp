use std::mem;

use futures::{
    executor::block_on,
    future::{select_all, BoxFuture},
    Future,
};

type MsgStreamFuture<Msg> =
    BoxFuture<'static, (Option<Box<dyn MsgStream<Msg>>>, Msg)>;

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

impl<T, Msg, F, Fut> MsgGen<T, Msg, F, Fut>
where
    F: Fn(T) -> Fut + Send,
    Fut: Future<Output = (Option<T>, Msg)> + Send + 'static,
{
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

pub trait MsgStream<Msg> {
    fn next_future(self: Box<Self>) -> MsgStreamFuture<Msg>;
}

pub struct Streams<Msg> {
    futures: Vec<MsgStreamFuture<Msg>>,
}

impl<Msg> Streams<Msg> {
    pub fn new() -> Self {
        Self { futures: vec![] }
    }

    pub fn wait_one(&mut self) -> Msg {
        let res = block_on(select_all(mem::take(&mut self.futures)));
        self.futures = res.2;
        if let Some(task) = res.0 .0 {
            self.futures.push(task.next_future());
        }
        res.0 .1
    }

    pub fn add(&mut self, f: Box<dyn MsgStream<Msg>>) {
        self.futures.push(f.next_future());
    }
}
