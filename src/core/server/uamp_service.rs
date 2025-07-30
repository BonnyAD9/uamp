use std::{borrow::Cow, convert::Infallible};

use futures::{
    StreamExt,
    stream::{self, BoxStream},
};
use http_body_util::StreamBody;
use hyper::{
    Method, Request, Response, Version,
    body::{Bytes, Frame, Incoming},
};
use pareg::FromArg;
use url::Url;

use crate::core::{AnyControlMsg, Error, Msg, Result, RtAndle};

type MyBody = StreamBody<BoxStream<'static, Result<Frame<Bytes>>>>;
type MyResponse = Response<MyBody>;

#[derive(Debug, Clone)]
pub struct UampService {
    rt: RtAndle,
}

impl UampService {
    pub fn new(rt: RtAndle) -> Self {
        Self { rt }
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
            _ => Err(Error::http(405, "Unknown method.")),
        }
    }

    async fn serve_get(&self, req: Request<Incoming>) -> Result<MyResponse> {
        match req.uri().path() {
            "/api/ctrl" => self.handle_instance_api(req).await,
            _ => Err(Error::http(404, "Unknown api endpoint.")),
        }
    }

    async fn handle_instance_api(
        &self,
        req: Request<Incoming>,
    ) -> Result<MyResponse> {
        let url = Url::parse(
            &("http://localhost".to_string()
                + req.uri().path_and_query().unwrap().as_str()),
        )?;
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
}

fn err_response(err: Error) -> MyResponse {
    let code = match err {
        Error::InvalidOperation(_)
        | Error::Pareg(_)
        | Error::SerdeJson(_)
        | Error::SerdeRmpDecode(_)
        | Error::ShellWords(_) => 400,
        Error::NotFound(_) => 404,
        _ => 500,
    };

    let msg = err.log().to_string();

    Response::builder()
        .status(code)
        .version(Version::HTTP_2)
        .body(string_body(msg))
        .expect("Failed to generate error response. This shouldn't happen.")
}

fn string_response(s: impl Into<Cow<'static, str>>) -> MyResponse {
    Response::builder()
        .status(200)
        .version(Version::HTTP_2)
        .header("Content-Type", "text/plain")
        .body(string_body(s))
        .expect("Failed to generate error response. This shouldn't happen.")
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
