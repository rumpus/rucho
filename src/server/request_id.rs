//! Request-ID middleware.
//!
//! Ensures every response carries an `X-Request-Id` header for correlation with
//! distributed-tracing systems. If the inbound request already carries a
//! non-blank `X-Request-Id` (Envoy / Kong Mesh (Kuma) sidecars use this header
//! natively, or an upstream client may set it), that value is propagated to the
//! response unchanged; otherwise a fresh UUID v4 is minted.
//!
//! The request is forwarded untouched, so echo endpoints (`/get`, `/headers`)
//! reflect exactly what the client sent. The response header is set only when a
//! handler has not already set one (e.g. `/response-headers`), so a handler's
//! deliberate value wins. When the header appears multiple times inbound, the
//! first value is used (`HeaderMap::get` semantics).
//!
//! Gated by the `request_id_enabled` config toggle (default on).

use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use http::header::HeaderValue;
use uuid::Uuid;

/// Canonical correlation-ID header name (lowercase, HTTP/2-safe).
const HEADER: &str = "x-request-id";

/// Middleware that ensures every response carries an `X-Request-Id` header.
///
/// Propagates a non-blank inbound `X-Request-Id` when present, otherwise mints a
/// UUID v4. A value a handler already set is left untouched; the request itself
/// is never modified.
pub async fn request_id_middleware(request: Request, next: Next) -> Response<Body> {
    // Reuse a non-blank inbound id (mesh/client correlation), else mint one.
    let request_id = request
        .headers()
        .get(HEADER)
        .filter(|value| value.as_bytes().iter().any(|b| !b.is_ascii_whitespace()))
        .cloned()
        .unwrap_or_else(new_request_id);

    let mut response = next.run(request).await;

    // Fill it in only when a handler hasn't already set one (e.g.
    // `/response-headers`), so a deliberate handler value wins.
    let headers = response.headers_mut();
    if !headers.contains_key(HEADER) {
        headers.insert(HEADER, request_id);
    }

    response
}

/// Generates a fresh request ID as a UUID v4 header value.
fn new_request_id() -> HeaderValue {
    // A UUID v4 string is ASCII hex + hyphens, so this never fails. (Inbound
    // CR/LF can't reach here either — httparse/http reject control bytes at
    // parse time — but this path only ever sees a freshly-minted UUID anyway.)
    HeaderValue::from_str(&Uuid::new_v4().to_string())
        .expect("UUID v4 string is always a valid header value")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use http::StatusCode;
    use tower::ServiceExt;

    /// A handler that deliberately sets its own `x-request-id` on the response.
    async fn preset() -> Response<Body> {
        let mut resp = Response::new(Body::from("preset"));
        resp.headers_mut()
            .insert(HEADER, HeaderValue::from_static("handler-set"));
        resp
    }

    /// A minimal app with the request-id layer over a plain and a preset route.
    fn app() -> Router {
        Router::new()
            .route("/", get(|| async { "ok" }))
            .route("/preset", get(preset))
            .layer(axum::middleware::from_fn(request_id_middleware))
    }

    #[tokio::test]
    async fn generates_uuid_v4_when_absent() {
        let resp = app()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let id = resp
            .headers()
            .get(HEADER)
            .expect("response must carry x-request-id")
            .to_str()
            .unwrap();
        let parsed = Uuid::parse_str(id).expect("response id must be a valid UUID");
        assert_eq!(parsed.get_version_num(), 4);
    }

    #[tokio::test]
    async fn propagates_inbound_id() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(HEADER, "client-correlation-123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            resp.headers().get(HEADER).unwrap(),
            "client-correlation-123"
        );
    }

    #[tokio::test]
    async fn mints_uuid_when_inbound_empty() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(HEADER, "")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let id = resp.headers().get(HEADER).unwrap().to_str().unwrap();
        assert!(!id.is_empty(), "empty inbound id must be replaced");
        Uuid::parse_str(id).expect("must mint a valid UUID when inbound is empty");
    }

    #[tokio::test]
    async fn mints_uuid_when_inbound_blank() {
        // A whitespace-only id is semantically empty and useless for tracing.
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(HEADER, "   ")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let id = resp.headers().get(HEADER).unwrap().to_str().unwrap();
        assert_ne!(id.trim(), "", "blank inbound id must be replaced");
        Uuid::parse_str(id).expect("must mint a valid UUID when inbound is blank");
    }

    #[tokio::test]
    async fn handler_set_value_is_preserved() {
        // A handler (e.g. /response-headers) that sets x-request-id must win;
        // the middleware must not clobber it.
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/preset")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            resp.headers().get(HEADER).unwrap(),
            "handler-set",
            "a handler-set x-request-id must not be overwritten"
        );
    }

    #[tokio::test]
    async fn ids_are_unique_per_request() {
        let first = app()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let second = app()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_ne!(
            first.headers().get(HEADER).unwrap(),
            second.headers().get(HEADER).unwrap(),
            "each generated request id must be distinct"
        );
    }
}
