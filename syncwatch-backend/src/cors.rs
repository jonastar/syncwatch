use axum::{
    body::{boxed, BoxBody, Empty},
    http::{header, HeaderMap, HeaderValue, Method, Request, Response},
};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct CorsLayer;

impl<S> Layer<S> for CorsLayer {
    type Service = CorsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CorsMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct CorsMiddleware<S> {
    inner: S,
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

impl<S, ReqBody> Service<Request<ReqBody>> for CorsMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<BoxBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // best practice is to clone the inner service like this
        // see https://github.com/tower-rs/tower/issues/547 for details
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let origin = req
            .headers()
            .get(header::ORIGIN)
            .map(|v| v.to_str().unwrap_or(""))
            .unwrap_or("")
            .to_owned();

        if matches!(req.method(), &Method::OPTIONS) {
            Box::pin(async move {
                let mut resp = Response::new(boxed(Empty::new()));

                insert_headers(&origin, resp.headers_mut());

                Ok(resp)
            })
        } else {
            Box::pin(async move {
                match inner.call(req).await {
                    Ok(mut resp) => {
                        insert_headers(&origin, resp.headers_mut());

                        Ok(resp)
                    }
                    Err(err) => Err(err),
                }
            })
        }
    }
}

fn insert_headers(host_base: &str, headers: &mut HeaderMap) {
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_str(host_base).unwrap(),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("*"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("*"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        HeaderValue::from_static("true"),
    );
}
