//! Correlated request logs, including cancellation before completion.
use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

pub(crate) async fn log_http_request(request: Request, next: Next) -> Response {
    let request_id = NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
    let mut log = HttpRequestLog {
        request_id,
        method: request.method().to_string(),
        uri: request.uri().to_string(),
        started: Instant::now(),
        finished: false,
    };
    tracing::info!(request_id, method = %log.method, uri = %log.uri, "http request started");

    let mut response = next.run(request).await;
    tracing::info!(
        request_id,
        method = %log.method,
        uri = %log.uri,
        status = response.status().as_u16(),
        elapsed_ms = log.started.elapsed().as_millis() as u64,
        "http request finished"
    );
    log.finished = true;
    response.headers_mut().insert(
        HeaderName::from_static("x-request-id"),
        HeaderValue::from_str(&request_id.to_string()).expect("numeric request id is a header"),
    );
    response
}

struct HttpRequestLog {
    request_id: u64,
    method: String,
    uri: String,
    started: Instant,
    finished: bool,
}

impl Drop for HttpRequestLog {
    fn drop(&mut self) {
        if !self.finished {
            tracing::warn!(
                request_id = self.request_id,
                method = %self.method,
                uri = %self.uri,
                elapsed_ms = self.started.elapsed().as_millis() as u64,
                "http request canceled before a response was produced"
            );
        }
    }
}
