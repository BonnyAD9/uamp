use std::mem;

use futures::{
    executor::block_on,
    future::{select_all, BoxFuture},
    Future,
};

type TaskFuture<Msg> = BoxFuture<'static, (Box<dyn TaskGen<Msg>>, Msg)>;

pub struct Task<T, Msg, F, Fut>
where
    F: Fn(T) -> Fut + Send + 'static,
    Fut: Future<Output = (T, Msg)> + Send + 'static,
    Msg: 'static,
    T: 'static,
{
    data: T,
    fun: F,
}

impl<T, Msg, F, Fut> Task<T, Msg, F, Fut>
where
    F: Fn(T) -> Fut + Send,
    Fut: Future<Output = (T, Msg)> + Send + 'static,
{
    pub fn new(data: T, fun: F) -> Self {
        Self { data, fun }
    }
}

impl<T, Msg, F, Fut> TaskGen<Msg> for Option<Task<T, Msg, F, Fut>>
where
    F: Fn(T) -> Fut + Send,
    Fut: Future<Output = (T, Msg)> + Send,
{
    fn task(&mut self) -> TaskFuture<Msg> {
        let Some(Task { data, fun }) = self.take() else {
            panic!(); // TODO
        };
        let f = fun(data);
        Box::pin(async move {
            let (data, m) = f.await;
            let slf: Box<dyn TaskGen<Msg>> =
                Box::new(Some(Task { data, fun }));
            (slf, m)
        })
    }
}

pub trait TaskGen<Msg> {
    fn task(&mut self) -> TaskFuture<Msg>;
}

pub struct Tasks<Msg> {
    futures: Vec<TaskFuture<Msg>>,
}

impl<Msg> Tasks<Msg> {
    pub fn new() -> Self {
        Self { futures: vec![] }
    }

    pub fn wait_one(&mut self) -> Msg {
        let mut res = block_on(select_all(mem::take(&mut self.futures)));
        self.futures = res.2;
        self.futures.push(res.0 .0.task());
        res.0 .1
    }

    pub fn add(&mut self, mut f: Box<dyn TaskGen<Msg>>) {
        self.futures.push(f.task());
    }
}
