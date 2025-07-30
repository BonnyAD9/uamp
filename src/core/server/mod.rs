mod uamp_service;
mod executor;

use std::net::SocketAddr;

use async_std::net::TcpListener;
use futures::{channel::{mpsc::UnboundedSender, oneshot}, executor::block_on};
use hyper::{server::conn::http2, service::service_fn};

use crate::{core::{config::Config, server::executor::Executor, Msg, Result, UampApp}, env::AppCtrl};

pub use self::uamp_service::*;

struct Server {
    sender: UnboundedSender<Msg>,
    listener: TcpListener,
}

impl UampApp {
    pub fn start_server(conf: &Config, ctrl: &mut AppCtrl, sender: UnboundedSender<Msg>) -> Result<oneshot::Sender<()>> {
        let server = Server::new(conf, sender)?;
        let (ssend, srecv) = oneshot::channel::<()>();
        ctrl.add_fn_stream((server, srecv), |(server, stop)| async move {
            if let Some((msg, stop)) = server.run(stop).await {
                (Some((server, stop)), Some(msg))
            } else {
                (None, None)
            }
        });
        Ok(ssend)
    }
}

impl Server {
    pub fn new(conf: &Config, sender: UnboundedSender<Msg>) -> Result<Self> {
        let listener = block_on(TcpListener::bind(format!("{}:{}", conf.server_address(), conf.port())))?;
        Ok(Self { sender, listener })
    }
    
    async fn run(&self, stop: oneshot::Receiver<()>) -> Option<(Msg, oneshot::Receiver<()>)> {
        // TODO: better error handling
        let (conn, _) = self.listener.accept().await.ok()?;
        self.sender.unbounded_send(Msg::fn_delegate(move |app, ctrl| {
            ctrl.add_once_stream((conn.clone(), app.sender.clone()), |(conn, sender)| async move {
                let service = UampService::new(sender.clone());
                let _ = http2::Builder::new(Executor::new(sender.clone()))
                    .serve_connection(conn, service_fn(async move |a| service.serve(a).await))
                    .await;
            });
            Ok(vec![])
        })).ok()?;
        todo!()
    }
}