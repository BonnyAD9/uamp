mod andle;
mod handle;
mod msg;

use futures::{StreamExt, stream};
use tokio::sync::mpsc;

use crate::env::streams::Streams;

pub use self::{andle::*, handle::*, msg::*};

pub fn make_rt<M, E: Send>() -> (Streams<Msg<M, E>>, Handle<M, E>) {
    let (send, recv) = mpsc::unbounded_channel::<Msg<M, E>>();
    let (asend, arecv) = mpsc::unbounded_channel::<Amsg<M, E>>();

    let mut streams = Streams::new();

    streams.add_stream(
        stream::unfold(recv, |mut recv| async move {
            recv.recv().await.map(|a| (a, recv))
        })
        .boxed_local(),
    );

    streams.add_stream(
        stream::unfold(arecv, |mut arecv| async move {
            arecv.recv().await.map(|a| (a.into(), arecv))
        })
        .boxed_local(),
    );

    (
        streams,
        Handle {
            local: send,
            thread: asend,
        },
    )
}
