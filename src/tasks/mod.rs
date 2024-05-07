use std::mem;

use futures::{
    executor::block_on,
    future::{select_all, BoxFuture},
    Future,
};

type TaskFuture<Msg> = BoxFuture<'static, (Option<Box<dyn TaskGen<Msg>>>, Msg)>;

pub struct Task<T, Msg, F, Fut>
where
    F: Fn(T) -> Fut + Send + 'static,
    Fut: Future<Output = (Option<T>, Msg)> + Send + 'static,
    Msg: 'static,
    T: 'static,
{
    data: T,
    fun: F,
}

impl<T, Msg, F, Fut> Task<T, Msg, F, Fut>
where
    F: Fn(T) -> Fut + Send,
    Fut: Future<Output = (Option<T>, Msg)> + Send + 'static,
{
    pub fn new(data: T, fun: F) -> Self {
        Self { data, fun }
    }
}

impl<T, Msg, F, Fut> TaskGen<Msg> for Task<T, Msg, F, Fut>
where
    F: Fn(T) -> Fut + Send,
    Fut: Future<Output = (Option<T>, Msg)> + Send,
    T: Send,
{
    fn task(mut self: Box<Self>) -> TaskFuture<Msg> {
        Box::pin(async move {
            let (data, m) = (self.fun)(self.data).await;
            if let Some(data) = data {
                *self = Task {
                    data,
                    fun: self.fun,
                };
                let slf: Box<dyn TaskGen<_>> = self;
                (Some(slf), m)
            } else {
                (None, m)
            }
        })
    }
}

pub trait TaskGen<Msg> {
    fn task(self: Box<Self>) -> TaskFuture<Msg>;
}

pub struct Tasks<Msg> {
    futures: Vec<TaskFuture<Msg>>,
}

impl<Msg> Tasks<Msg> {
    pub fn new() -> Self {
        Self { futures: vec![] }
    }

    pub fn wait_one(&mut self) -> Msg {
        let res = block_on(select_all(mem::take(&mut self.futures)));
        self.futures = res.2;
        if let Some(task) = res.0 .0 {
            self.futures.push(task.task());
        }
        res.0 .1
    }

    pub fn add(&mut self, f: Box<dyn TaskGen<Msg>>) {
        self.futures.push(f.task());
    }
}
