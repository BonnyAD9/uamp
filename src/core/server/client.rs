use anyhow::anyhow;
use http_body_util::{BodyExt, Full};
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    client::conn::http1::{self, SendRequest},
};
use hyper_util::rt::TokioIo;
use tokio::{net::TcpStream, task::JoinHandle};
use url::Url;

use crate::core::{
    AnyControlMsg, Error, Result,
    library::Song,
    log_err,
    query::Query,
    server::{Info, RepMsg},
};

pub struct Client {
    sender: SendRequest<Full<Bytes>>,
    _handle: JoinHandle<()>,
    authority: String,
}

impl Client {
    pub async fn connect(address: String) -> Result<Self> {
        let stream = TcpStream::connect(&address).await?;
        let (sender, conn) = http1::handshake(TokioIo::new(stream)).await?;
        let handle = tokio::spawn(async move {
            log_err("Connection failed.", conn.await);
        });
        Ok(Self {
            sender,
            _handle: handle,
            authority: address,
        })
    }

    pub async fn send_ctrl(&mut self, msgs: &[AnyControlMsg]) -> Result<()> {
        let url = Url::parse_with_params(
            &("http://".to_string() + &self.authority + "/api/ctrl"),
            msgs.iter().map(ctrl_to_query),
        )?;

        let req = Request::builder()
            .uri(url.to_string())
            .header(hyper::header::HOST, url.authority())
            .body(Full::new(Bytes::new()))?;

        let res = self.sender.send_request(req).await?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(Error::http(
                res.status().as_u16(),
                body_to_string(res).await?,
            ))
        }
    }

    pub async fn req_info(&mut self, b: usize, a: usize) -> Result<Box<Info>> {
        let url = Url::parse_with_params(
            &("http://".to_string() + &self.authority + "/api/req"),
            &[("nfo", format!("-{a}..{b}"))],
        )?;

        let req = Request::builder()
            .uri(url.to_string())
            .header(hyper::header::HOST, url.authority())
            .body(Full::new(Bytes::new()))?;

        let res = self.sender.send_request(req).await?;

        if !res.status().is_success() {
            return Err(Error::http(
                res.status().as_u16(),
                body_to_string(res).await?,
            ));
        }

        let mut res =
            serde_json::from_slice::<Vec<RepMsg>>(&body_to_vec(res).await?)?;

        if res.len() != 1 {
            return Err(Error::Unexpected(
                "Response didn't contain correct amount of data.".into(),
            ));
        }

        let res = res.pop().unwrap();
        match res {
            RepMsg::Info(i) => Ok(i),
            RepMsg::Error(e) => Err(Error::Other(anyhow!(e).into())),
            _ => Err(Error::Unexpected("Unexpected response.".into())),
        }
    }

    pub async fn req_query(&mut self, q: &Query) -> Result<Vec<Song>> {
        let url = Url::parse_with_params(
            &("http://".to_string() + &self.authority + "/api/req"),
            &[("l", q.to_string())],
        )?;

        let req = Request::builder()
            .uri(url.to_string())
            .header(hyper::header::HOST, url.authority())
            .body(Full::new(Bytes::new()))?;

        let res = self.sender.send_request(req).await?;

        if !res.status().is_success() {
            return Err(Error::http(
                res.status().as_u16(),
                body_to_string(res).await?,
            ));
        }

        let mut res =
            serde_json::from_slice::<Vec<RepMsg>>(&body_to_vec(res).await?)?;

        if res.len() != 1 {
            return Err(Error::Unexpected(
                "Response didn't contain correct amount of data.".into(),
            ));
        }

        let res = res.pop().unwrap();
        match res {
            RepMsg::Query(s) => Ok(s),
            RepMsg::Error(e) => Err(Error::Other(anyhow!(e).into())),
            _ => Err(Error::Unexpected("Unexpected response.".into())),
        }
    }
}

fn ctrl_to_query(msg: &AnyControlMsg) -> (String, String) {
    let msg = msg.to_string();
    if let Some((pre, post)) = msg.split_once('=') {
        (pre.to_string(), post.to_string())
    } else {
        (msg, "".to_string())
    }
}

async fn body_to_string(mut body: Response<Incoming>) -> Result<String> {
    let mut res = String::new();
    while let Some(next) = body.frame().await {
        let frame = next?;
        if let Some(chunk) = frame.data_ref() {
            res += &String::from_utf8_lossy(chunk);
        }
    }
    Ok(res)
}

async fn body_to_vec(mut body: Response<Incoming>) -> Result<Vec<u8>> {
    let mut res = vec![];
    while let Some(next) = body.frame().await {
        let frame = next?;
        if let Some(chunk) = frame.data_ref() {
            res.extend(chunk);
        }
    }
    Ok(res)
}
