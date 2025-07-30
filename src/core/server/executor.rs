use std::sync::Mutex;

use futures::{channel::mpsc::UnboundedSender};

use crate::core::Msg;

#[derive(Clone)]
pub struct Executor {
    sender: UnboundedSender<Msg>,
}

impl Executor {
    pub fn new(sender: UnboundedSender<Msg>) -> Self {
        Self { sender }
    }
}

impl<F> hyper::rt::Executor<F> for Executor
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        let fut = Mutex::new(Some(fut));
        // TODO report error
        _ = self.sender.unbounded_send(Msg::fn_delegate(move |_, ctrl| {
            let Some(fut) = fut.lock().unwrap().take() else {
                return Ok(vec![]); // TODO report error
            };
            ctrl.add_once_stream(fut, |fut| async move {
                fut.await;
            });
            
            Ok(vec![])
        }));
    }
}