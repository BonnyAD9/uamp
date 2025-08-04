use std::{
    borrow::Cow,
    convert::Infallible,
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use futures::{
    StreamExt,
    stream::{self, BoxStream},
};
use http_body_util::StreamBody;
use hyper::{
    Method, Request, Response, Uri,
    body::{Bytes, Frame, Incoming},
};
use pareg::FromArg;
use serde::Serialize;
use tokio::sync::broadcast;
use url::Url;

use crate::core::{
    AnyControlMsg, Error, Msg, Result, RtAndle, UampApp, config,
    library::Song,
    query::Query,
    server::{Info, RepMsg, ReqMsg, SubMsg, sse_service::SseService},
};

type MyBody = StreamBody<BoxStream<'static, Result<Frame<Bytes>>>>;
type MyResponse = Response<MyBody>;

#[derive(Debug, Clone)]
pub struct UampService {
    rt: RtAndle,
    brcs: broadcast::WeakSender<SubMsg>,
    app_path: PathBuf,
}

impl UampService {
    pub fn new(
        rt: RtAndle,
        brcs: broadcast::WeakSender<SubMsg>,
        app_path: PathBuf,
    ) -> Self {
        Self { rt, brcs, app_path }
    }

    pub async fn serve(
        &self,
        req: Request<Incoming>,
    ) -> std::result::Result<MyResponse, Infallible> {
        match self.serve_inner(req).await {
            Ok(r) => Ok(r),
            Err(e) => Ok(err_response(e)),
        }
    }

    async fn serve_inner(&self, req: Request<Incoming>) -> Result<MyResponse> {
        match *req.method() {
            Method::GET => self.serve_get(req).await,
            _ => Err(Error::http(405, "Unknown method.".to_string())),
        }
    }

    async fn serve_get(&self, req: Request<Incoming>) -> Result<MyResponse> {
        let referer = get_referrer_path(&req);
        match (req.uri().path(), referer.as_deref()) {
            ("/api/ctrl", _) => self.handle_ctrl_api(req).await,
            ("/api/req", _) => self.handle_req_api(req).await,
            ("/api/sub", _) => self.handle_sub_api(req).await,
            (v, _) if v.starts_with("/app") => {
                self.handle_app(v.strip_prefix("/app").unwrap()).await
            }
            (v, Some("/app" | "/app/")) => self.handle_app(v).await,
            _ => Err(Error::http(404, "Unknown api endpoint.".to_string())),
        }
    }

    async fn handle_ctrl_api(
        &self,
        req: Request<Incoming>,
    ) -> Result<MyResponse> {
        let url = uri_to_url(req.uri())?;
        let mut msgs: Vec<Msg> = vec![];
        // This is kind of dirty solution, but it works.
        let mut buf = String::new();
        for (k, v) in url.query_pairs() {
            buf.clear();
            buf += &k;
            if !v.is_empty() {
                buf.push('=');
                buf += &v;
            }
            msgs.push(AnyControlMsg::from_arg(&buf)?.into());
        }

        self.rt.msgs(msgs);

        Ok(string_response("Success!"))
    }

    async fn handle_req_api(
        &self,
        req: Request<Incoming>,
    ) -> Result<MyResponse> {
        let url = uri_to_url(req.uri())?;
        let mut reply = vec![];

        let mut any_good = false;
        let mut first_bad = None;

        for (k, v) in url.query_pairs() {
            let res = match self.make_req(&k, &v).await {
                Ok(r) => {
                    any_good = true;
                    r
                }
                Err(mut e) => {
                    e = e.log();
                    let res = RepMsg::Error(e.to_string());
                    first_bad = Some(e);
                    res
                }
            };

            reply.push(res);
        }

        if let Some(err) = first_bad
            && !any_good
        {
            return Err(err);
        }

        json_response(&reply)
    }

    async fn handle_sub_api(
        &self,
        _: Request<Incoming>,
    ) -> Result<MyResponse> {
        let Some(s) = self.brcs.upgrade() else {
            return Err(Error::http(500, "No event source.".into()));
        };
        let srv = SseService::new(s.subscribe(), self.rt.clone());
        Ok(sse_response(srv))
    }

    async fn handle_app(&self, mut path: &str) -> Result<MyResponse> {
        path = path.strip_prefix("/").unwrap_or(path);
        if path.is_empty() {
            path = "index.html";
        }
        let path = self.app_path.join(path);
        // TODO: disallow .. in path
        file_response(path)
    }

    async fn make_req(&self, k: &str, v: &str) -> Result<RepMsg> {
        match ReqMsg::from_kv(k, v)? {
            ReqMsg::Info(b, a) => self.handle_info_req(b, a).await,
            ReqMsg::Query(q) => self.handle_query_req(q).await,
        }
    }

    async fn handle_info_req(&self, b: usize, a: usize) -> Result<RepMsg> {
        self.rt
            .request(move |app, _| app.info_response(b, a))
            .await
            .map(RepMsg::Info)
    }

    async fn handle_query_req(&self, q: Query) -> Result<RepMsg> {
        self.rt
            .request(move |app, _| app.query_response(&q))
            .await?
            .map(RepMsg::Query)
    }
}

impl UampApp {
    fn info_response(&mut self, b: usize, a: usize) -> Box<Info> {
        let idx = self.player.playlist().current_idx();
        let (before, after) = if let Some(idx) = idx {
            let start = idx.saturating_sub(b);
            let end = (idx + a + 1).min(self.player.playlist().len());
            (
                self.player.playlist()[start..idx]
                    .iter()
                    .map(|i| self.library[*i].clone())
                    .collect(),
                self.player.playlist()[idx + 1..end]
                    .iter()
                    .map(|i| self.library[*i].clone())
                    .collect(),
            )
        } else {
            (vec![], vec![])
        };

        Box::new(Info {
            version: config::VERSION_STR.to_owned(),
            now_playing: self
                .player
                .now_playing()
                .map(|i| self.library[i].clone()),
            playlist_len: self.player.playlist().len(),
            playlist_pos: self.player.playlist().get_pos(),
            is_playing: self.player.is_playing(),
            volume: self.player.volume(),
            mute: self.player.mute(),
            timestamp: self.player.timestamp(),
            before,
            after,
            playlist_stack: self
                .player
                .playlist_stack()
                .iter()
                .map(|p| (p.current_idx(), p.len()))
                .collect(),
            playlist_end: self
                .player
                .playlist()
                .on_end
                .as_ref()
                .or(self.config.default_playlist_end_action().as_ref())
                .cloned(),
            playlist_add_policy: self.player.playlist().add_policy,
        })
    }

    fn query_response(&mut self, query: &Query) -> Result<Vec<Song>> {
        query.clone_songs(
            &self.library,
            self.config.simple_sorting(),
            &self.player,
        )
    }
}

fn err_response(err: Error) -> MyResponse {
    let code = match err {
        Error::InvalidOperation(_)
        | Error::Pareg(_)
        | Error::SerdeJson(_)
        | Error::ShellWords(_) => 400,
        Error::NotFound(_) => 404,
        Error::Http(_, c) => c,
        _ => 500,
    };

    let msg = err.log().to_string();

    Response::builder()
        .status(code)
        .body(string_body(msg))
        .expect("Failed to generate error response. This shouldn't happen.")
}

fn string_response(s: impl Into<Cow<'static, str>>) -> MyResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain")
        .body(string_body(s))
        .expect("Failed to generate string response. This shouldn't happen.")
}

fn json_response(s: &impl Serialize) -> Result<MyResponse> {
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(json_body(s)?)
        .expect("Failed to generate json response. This shouldn't happen."))
}

fn sse_response(s: SseService) -> MyResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(StreamBody::new(
            stream::unfold(s, |mut s| async move {
                let data = Ok(Frame::data(s.next().await.into()));
                Some((data, s))
            })
            .boxed(),
        ))
        .expect("Failet to generate sse response. This shouldn't happen.")
}

fn file_response(p: impl AsRef<Path>) -> Result<MyResponse> {
    let mut res = vec![];
    BufReader::new(File::open(p.as_ref())?).read_to_end(&mut res)?;
    let mime = mime_guess::from_path(p).first_or_octet_stream();
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", mime.essence_str())
        .body(byte_body(res))
        .expect("Failed to generate file response. This shouldn't happen."))
}

fn string_body(s: impl Into<Cow<'static, str>>) -> MyBody {
    let s = s.into();
    StreamBody::new(
        stream::once(async move {
            match s {
                Cow::Owned(s) => Ok(Frame::data(s.into())),
                Cow::Borrowed(s) => Ok(Frame::data(s.into())),
            }
        })
        .boxed(),
    )
}

fn json_body(s: &impl Serialize) -> Result<MyBody> {
    let data = serde_json::to_vec(s)?;
    Ok(byte_body(data))
}

fn byte_body(b: Vec<u8>) -> MyBody {
    StreamBody::new(
        stream::once(async move { Ok(Frame::data(b.into())) }).boxed(),
    )
}

fn uri_to_url(uri: &Uri) -> Result<Url> {
    Ok(Url::parse(
        &("http://dummy".to_string() + uri.path_and_query().unwrap().as_str()),
    )?)
}

fn get_referrer_path(req: &Request<Incoming>) -> Option<String> {
    let r = req.headers().get("Referer")?.to_str().ok()?;
    let url = Url::parse(r).ok()?;
    Some(url.path().to_string())
}
