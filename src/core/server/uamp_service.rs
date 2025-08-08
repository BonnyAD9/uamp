use std::{borrow::Cow, convert::Infallible, env, io::ErrorKind, path::Path};

use const_format::concatc;
use futures::{
    StreamExt,
    stream::{self, BoxStream},
};
use http_body_util::{BodyExt, StreamBody};
use hyper::{
    Method, Request, Response, Uri,
    body::{Bytes, Frame, Incoming},
};
use pareg::{ArgInto, FromArg};
use serde::Serialize;
use tokio::{
    fs::{self, File},
    io::AsyncRead,
};
use tokio_tar::Archive;
use tokio_util::codec::{BytesCodec, FramedRead};
use url::Url;

use crate::core::{
    AnyControlMsg, Error, Msg, Result, RtAndle, UampApp,
    config::{self, CacheSize},
    library::{Song, img_lookup::lookup_image_path_rt_thread},
    query::Query,
    server::{Info, RepMsg, ReqMsg, ServerData, sse_service::SseService},
};

type MyBody = StreamBody<BoxStream<'static, Result<Frame<Bytes>>>>;
type MyResponse = Response<MyBody>;

#[derive(Debug, Clone)]
pub struct UampService {
    rt: RtAndle,
    data: ServerData,
}

pub const SERVER_HEADER: &str = concatc!(
    config::APP_ID,
    "/",
    config::VERSION_STR,
    " (",
    env::consts::OS,
    ")"
);

/// Maximum limit for the amount of cumulative accepted data.
pub const MAX_ACCEPT_LENGTH: usize = 1024 * 1024; // 1 MiB

impl UampService {
    pub fn new(rt: RtAndle, data: ServerData) -> Self {
        Self { rt, data }
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
            Method::POST => self.serve_post(req).await,
            _ => Err(Error::http(405, "Unknown method.".to_string())),
        }
    }

    async fn serve_get(&self, req: Request<Incoming>) -> Result<MyResponse> {
        match req.uri().path() {
            "/api/ctrl" => self.handle_ctrl_api(req).await,
            "/api/req" => self.handle_req_api(req).await,
            "/api/sub" => self.handle_sub_api(req).await,
            "/api/marco" => Ok(string_response("polo")),
            "/api/img" => self.handle_img_api(req).await,
            v if v.starts_with("/app/") || v == "/app" => {
                self.handle_app(v.strip_prefix("/app").unwrap()).await
            }
            _ => Err(Error::http(404, "Unknown GET endpoint.".to_string())),
        }
    }

    async fn serve_post(&self, req: Request<Incoming>) -> Result<MyResponse> {
        match req.uri().path() {
            "/api/ctrl" => self.handle_ctrl_post_api(req).await,
            _ => Err(Error::http(404, "Unknown POST endpoint.".to_string())),
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

        self.rt.msgs_result(msgs).await?;

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
        let Some(s) = self.data.make_reciever() else {
            return Err(Error::http(204, "No event source.".into()));
        };
        let srv = SseService::new(s, self.rt.clone());
        Ok(sse_response(srv))
    }

    async fn handle_img_api(
        &self,
        req: Request<Incoming>,
    ) -> Result<MyResponse> {
        let url = uri_to_url(req.uri())?;
        let mut album = None;
        let mut artist = None;
        let mut size: Option<CacheSize> = None;
        let mut or = None;

        for (k, v) in url.query_pairs() {
            match k.as_ref() {
                "artist" => artist = Some(v.into_owned()),
                "album" => album = Some(v.into_owned()),
                "size" => size = Some(v.arg_into()?),
                "or" => or = Some(v.into_owned()),
                _ => {}
            };
        }
        let Some(album) = album else {
            return Err(Error::http(400, "Missing album in query.".into()));
        };

        let Some(artist) = artist else {
            return Err(Error::http(400, "Missing artist in query.".into()));
        };

        let cache = self.data.cache.read().unwrap().clone();

        let res = lookup_image_path_rt_thread(
            self.rt.clone(),
            cache,
            artist,
            album,
            size.unwrap_or_default(),
        )
        .await?;

        match (res, or) {
            (Ok(r), _) => file_response(r).await,
            (Err(_), Some(o)) => Ok(redirect_response(&o, false)),
            (Err(e), _) => Err(e),
        }
    }

    async fn handle_app(&self, path: &str) -> Result<MyResponse> {
        let app_path = self.data.client.read().unwrap().clone();
        if fs::metadata(&app_path).await?.is_dir() {
            self.handle_app_dir(&app_path, path).await
        } else {
            self.handle_app_tar(&app_path, path).await
        }
    }

    async fn handle_ctrl_post_api(
        &self,
        mut req: Request<Incoming>,
    ) -> Result<MyResponse> {
        let len = req
            .headers()
            .get("Content-Length")
            .and_then(|l| l.to_str().ok())
            .and_then(|a| a.parse().ok());
        let mut str = req.body_mut().into_data_stream();

        let mut data = if let Some(len) = len {
            if len > MAX_ACCEPT_LENGTH {
                return Err(Error::http(413, "Too much data.".to_string()));
            }
            Vec::with_capacity(len)
        } else {
            vec![]
        };

        while data.len() <= MAX_ACCEPT_LENGTH
            && let Some(frame) = str.next().await
        {
            let frame = frame?;
            data.extend(frame);
        }

        if data.len() > MAX_ACCEPT_LENGTH {
            return Err(Error::http(413, "Too much data.".to_string()));
        }

        let msg = serde_json::from_slice(&data)?;

        self.rt.msg_result(Msg::IdControl(msg)).await?;

        Ok(string_response("Success!"))
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

    async fn handle_app_dir(
        &self,
        app_path: &Path,
        mut path: &str,
    ) -> Result<MyResponse> {
        path = path.strip_prefix("/").unwrap_or(path);
        let os_path = Path::new(&path);
        if os_path.components().any(|c| c.as_os_str() == "..") {
            return Err(Error::http(
                403,
                "Relative `..` is disallowed.".into(),
            ));
        }
        let os_path = app_path.join(path);
        if fs::metadata(&os_path).await?.is_dir() {
            Ok(redirect_response(
                &path_join(["/app", path, "index.html"]),
                true,
            ))
        } else {
            file_response(os_path).await
        }
    }

    async fn handle_app_tar(
        &self,
        app_path: &Path,
        mut path: &str,
    ) -> Result<MyResponse> {
        path = path.strip_prefix('/').unwrap_or(path);
        let sec_path = path_join([path, "index.html"]);

        let mut archive = Archive::new(File::open(app_path).await?);
        let mut entry = None;
        let mut entries = archive.entries()?;
        while let Some(e) = entries.next().await {
            let e = e?;
            let epath = e.path()?;
            if epath == Path::new(path) {
                entry = Some(e);
                break;
            }
            dbg!(&epath);
            if epath == Path::new(sec_path.as_str()) {
                return Ok(redirect_response(
                    &path_join(["/app", &sec_path]),
                    true,
                ));
            }
        }

        let Some(file) = entry else {
            return Err(Error::http(404, "No such file.".into()));
        };

        let mime = mime_guess::from_path(file.path()?).first_or_octet_stream();

        Ok(reader_response(file, mime.essence_str()))
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
        Error::Io(ref e) if e.inner().kind() == ErrorKind::NotFound => 404,
        Error::Http(_, c) => c,
        _ => 500,
    };

    let msg = err.log().to_string();

    Response::builder()
        .status(code)
        .header("Content-Type", "text/plain; charset=UTF-8")
        .header("Server", SERVER_HEADER)
        .body(string_body(msg))
        .expect("Failed to generate error response. This shouldn't happen.")
}

fn string_response(s: impl Into<Cow<'static, str>>) -> MyResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain; charset=UTF-8")
        .header("Server", SERVER_HEADER)
        .body(string_body(s))
        .expect("Failed to generate string response. This shouldn't happen.")
}

fn json_response(s: &impl Serialize) -> Result<MyResponse> {
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Server", SERVER_HEADER)
        .body(json_body(s)?)
        .expect("Failed to generate json response. This shouldn't happen."))
}

fn sse_response(s: SseService) -> MyResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .header("Server", SERVER_HEADER)
        .body(StreamBody::new(
            stream::unfold(s, |mut s| async move {
                let data = Ok(Frame::data(s.next().await?.into()));
                Some((data, s))
            })
            .boxed(),
        ))
        .expect("Failet to generate sse response. This shouldn't happen.")
}

async fn file_response(p: impl AsRef<Path>) -> Result<MyResponse> {
    let r = File::open(p.as_ref()).await?;
    let mime = mime_guess::from_path(p).first_or_octet_stream();
    Ok(reader_response(r, mime.essence_str()))
}

fn reader_response(
    r: impl AsyncRead + Unpin + Send + 'static,
    mime: &str,
) -> MyResponse {
    let fr = FramedRead::new(r, BytesCodec::new());
    Response::builder()
        .status(200)
        .header("Content-Type", mime)
        .header("Server", SERVER_HEADER)
        .body(StreamBody::new(
            stream::unfold(fr, |mut fr| async move {
                let n = fr
                    .next()
                    .await?
                    .map(|a| Frame::data(a.into()))
                    .map_err(|e| e.into());
                Some((n, fr))
            })
            .boxed(),
        ))
        .expect("Failed to generate reader response. This shouldn't happen.")
}

fn redirect_response(path: &str, permanent: bool) -> MyResponse {
    Response::builder()
        .status(if permanent { 301 } else { 302 })
        .header("Location", path)
        .header("Server", SERVER_HEADER)
        .body(empty_body())
        .expect("Failed to generate redirect response. This shouldn't happen.")
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

fn empty_body() -> MyBody {
    StreamBody::new(stream::empty().boxed())
}

fn uri_to_url(uri: &Uri) -> Result<Url> {
    Ok(Url::parse(
        &("http://dummy".to_string() + uri.path_and_query().unwrap().as_str()),
    )?)
}

fn path_join<S: AsRef<str>>(paths: impl IntoIterator<Item = S>) -> String {
    let mut it = paths.into_iter();
    let Some(p) = it.next() else {
        return String::new();
    };

    let mut res = p.as_ref().to_string();
    for p in it {
        let s = p.as_ref();
        if s.is_empty() {
            continue;
        }
        match (res.ends_with('/'), s.starts_with('/')) {
            (true, true) => res += &s[1..],
            (false, false) => {
                res.push('/');
                res += s;
            }
            _ => res += s,
        }
    }

    res
}
