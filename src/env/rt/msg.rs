use futures::stream::LocalBoxStream;
use tokio::sync::oneshot;

pub enum Msg<M, E> {
    Msg(Vec<M>, Option<oneshot::Sender<Result<(), E>>>),
    AddStream(LocalBoxStream<'static, Self>),
}
