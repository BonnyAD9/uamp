use futures::{
    Stream,
    stream::{self, LocalBoxStream, StreamExt},
};
use tokio::task::JoinHandle;

use crate::env::rt;

use super::Command;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Interface for performing actions in the environment wher UampApp runs.
#[derive(Debug)]
pub struct AppCtrl<'a, M: 'static, E: 'static> {
    commands: &'a mut Vec<Command<M, E>>,
}

impl<'a, M, E> AppCtrl<'a, M, E> {
    /// Creates new app control.
    pub fn new(commands: &'a mut Vec<Command<M, E>>) -> Self {
        Self { commands }
    }

    /// Add command to execute.
    pub fn add(&mut self, cmd: Command<M, E>) {
        self.commands.push(cmd);
    }

    /// Request the app to exit.
    pub fn exit(&mut self) {
        self.add(Command::Exit)
    }

    pub fn stream_rtx(&mut self, s: LocalBoxStream<'static, rt::Msg<M, E>>) {
        self.add(Command::AddStrem(s));
    }

    pub fn stream_rt(
        &mut self,
        s: impl Stream<Item = rt::Msg<M, E>> + 'static,
    ) {
        self.stream_rtx(s.boxed_local());
    }

    pub fn streams(&mut self, s: impl Stream<Item = Vec<M>> + 'static) {
        self.stream_rt(s.map(|a| rt::Msg::Msg(a, None)))
    }

    pub fn stream(&mut self, s: impl Stream<Item = M> + 'static) {
        self.streams(s.map(|a| vec![a]))
    }

    pub fn unfold_rt<
        T: 'static,
        F: Future<Output = Option<(rt::Msg<M, E>, T)>> + 'static,
    >(
        &mut self,
        d: T,
        f: impl FnMut(T) -> F + 'static,
    ) {
        self.stream_rt(stream::unfold(d, f))
    }

    pub fn unfold<T: 'static, F: Future<Output = Option<(M, T)>> + 'static>(
        &mut self,
        d: T,
        f: impl FnMut(T) -> F + 'static,
    ) {
        self.stream(stream::unfold(d, f))
    }

    pub fn task_rt(
        &mut self,
        f: impl Future<Output = rt::Msg<M, E>> + 'static,
    ) {
        self.unfold_rt(Some(f), |f| async move {
            if let Some(f) = f {
                Some((f.await, None))
            } else {
                None
            }
        })
    }

    pub fn tasks(&mut self, f: impl Future<Output = Vec<M>> + 'static) {
        self.task_rt(async move { rt::Msg::Msg(f.await, None) })
    }

    pub fn task(&mut self, f: impl Future<Output = M> + 'static) {
        self.tasks(async move { vec![f.await] })
    }

    pub fn spawn<T: 'static>(
        &mut self,
        f: impl Future<Output = T> + 'static,
    ) -> JoinHandle<T> {
        tokio::task::spawn_local(f)
    }
}
