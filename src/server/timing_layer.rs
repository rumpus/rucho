//! Request timing middleware layer.
//!
//! Captures the start time of each request (exposed to handlers via the
//! [`RequestTiming`] extension — e.g. for `duration_ms` in echo bodies) and
//! sets an `X-Response-Time` header on the response reflecting handler
//! processing time. The header lets a client compare upstream-measured latency
//! against a gateway's own measurement.

use axum::{body::Body, extract::Request, http::HeaderValue, middleware::Next, response::Response};

use crate::utils::timing::RequestTiming;

/// Header name carrying the measured response time (e.g. `1.234ms`).
const RESPONSE_TIME_HEADER: &str = "x-response-time";

/// Middleware that records request start time and stamps `X-Response-Time`.
///
/// Inserts a [`RequestTiming`] into the request extensions (so handlers can read
/// the elapsed time), then, after the inner handler returns, sets an
/// `X-Response-Time: <ms>ms` header on the response. This measures the inner
/// processing time (handler plus any inner middleware such as chaos), matching
/// the `duration_ms` value echo handlers report in their JSON body.
pub async fn timing_middleware(mut request: Request, next: Next) -> Response<Body> {
    let timing = RequestTiming::now();
    request.extensions_mut().insert(timing);

    let mut response = next.run(request).await;

    // `{:.3}ms` is always valid ASCII, so building the header never fails.
    if let Ok(value) = HeaderValue::from_str(&format!("{:.3}ms", timing.elapsed_ms())) {
        response.headers_mut().insert(RESPONSE_TIME_HEADER, value);
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use tower::ServiceExt;

    #[tokio::test]
    async fn sets_x_response_time_header() {
        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(timing_middleware));

        let resp = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let value = resp
            .headers()
            .get(RESPONSE_TIME_HEADER)
            .expect("response must carry x-response-time")
            .to_str()
            .unwrap();
        assert!(
            value.ends_with("ms"),
            "expected a <n>ms value, got: {value}"
        );
        let ms: f64 = value
            .trim_end_matches("ms")
            .parse()
            .expect("the numeric prefix must parse as f64 milliseconds");
        assert!(ms >= 0.0, "elapsed time cannot be negative");
    }
}
