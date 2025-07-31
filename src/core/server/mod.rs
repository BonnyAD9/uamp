mod info;
mod rep_msg;
mod req_msg;
mod snd_msg;
mod uamp_service;

use futures::executor::block_on;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

use crate::core::{
    AppCtrl, Error, Job, JobMsg, Msg, Result, RtHandle, UampApp,
    config::Config, log_err,
};

pub mod client;

pub use self::{info::*, rep_msg::*, req_msg::*, snd_msg::*, uamp_service::*};

struct Server {
    rt: RtHandle,
    listener: TcpListener,
}

impl UampApp {
    pub fn start_server(&mut self, ctrl: &mut AppCtrl) -> Result<()> {
        if self.jobs.is_running(Job::SERVER) {
            return Err(
                Error::invalid_operation().msg("Server is already running.")
            );
        }

        let cancel = CancellationToken::new();
        let server = Server::new(&self.config, self.rt.clone())?;
        let token = cancel.clone();
        ctrl.task(
            async move { Msg::Job(JobMsg::Server(server.run(token).await)) },
        );
        self.jobs.run(Job::SERVER);
        self.jobs.server = Some(cancel);

        Ok(())
    }
}

impl Server {
    pub fn new(conf: &Config, rt: RtHandle) -> Result<Self> {
        let listener = block_on(TcpListener::bind(format!(
            "{}:{}",
            conf.server_address(),
            conf.port()
        )))?;
        Ok(Self { rt, listener })
    }

    async fn run(&self, stop: CancellationToken) -> Result<()> {
        loop {
            let (conn, _) = tokio::select!(
                _ = stop.cancelled() => break,
                res = self.listener.accept() => {
                    let Some(val) = log_err("Failed to accept.", res) else {
                        continue;
                    };
                    val
                }
            );

            let service = UampService::new(self.rt.andle());
            self.rt.spawn(http1::Builder::new().serve_connection(
                TokioIo::new(conn),
                service_fn(move |a| {
                    let s = service.clone();
                    async move { s.serve(a).await }
                }),
            ));
        }

        Ok(())
    }
}
