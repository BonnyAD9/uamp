use futures::{
    StreamExt,
    stream::{BoxStream, LocalBoxStream},
};
use tokio::sync::oneshot;

pub enum Msg<M: 'static, E: 'static> {
    Msg(Vec<M>, Option<oneshot::Sender<Result<(), E>>>),
    AddStream(LocalBoxStream<'static, Self>),
}

pub enum Amsg<M, E> {
    Msg(Vec<M>, Option<oneshot::Sender<Result<(), E>>>),
    AddStream(BoxStream<'static, Self>),
}

impl<M, E> From<Amsg<M, E>> for Msg<M, E> {
    fn from(value: Amsg<M, E>) -> Self {
        match value {
            Amsg::Msg(m, r) => Msg::Msg(m, r),
            Amsg::AddStream(s) => {
                Msg::AddStream(s.map(|a| a.into()).boxed_local())
            }
        }
    }
}
