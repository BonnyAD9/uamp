mod info;
mod rep_msg;
mod req_msg;
mod snd_msg;
mod sse_service;
mod sub;
mod sub_msg;
mod uamp_service;

use std::path::PathBuf;

use futures::executor::block_on;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, sync::broadcast};
use tokio_util::sync::CancellationToken;

use crate::core::{
    AppCtrl, Error, Job, JobMsg, Msg, Result, RtHandle, UampApp,
    config::Config, log_err,
};

pub mod client;

pub use self::{
    info::*, rep_msg::*, req_msg::*, snd_msg::*, sub_msg::*, uamp_service::*,
};

const MAX_BROADCAST_CAPACITY: usize = 16;

struct Server {
    rt: RtHandle,
    listener: TcpListener,
    app_path: PathBuf,
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
        let (sub_broadcast, _) = broadcast::channel(MAX_BROADCAST_CAPACITY);
        let brcs = sub_broadcast.clone().downgrade();
        ctrl.task(async move {
            Msg::Job(JobMsg::Server(server.run(brcs, token).await))
        });
        self.jobs.run(Job::SERVER);
        self.jobs.server = Some((sub_broadcast, cancel));

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
        Ok(Self {
            rt,
            listener,
            app_path: conf.http_client().clone(),
        })
    }

    async fn run(
        &self,
        brcs: broadcast::WeakSender<SubMsg>,
        stop: CancellationToken,
    ) -> Result<()> {
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

            let service = UampService::new(
                self.rt.andle(),
                brcs.clone(),
                self.app_path.clone(),
            );
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
