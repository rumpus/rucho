//! Bytes endpoint — returns N random bytes as `application/octet-stream`.
//!
//! Provides a controllable upstream emitting arbitrary-sized binary bodies,
//! useful for exercising gateway proxy behavior: response buffering, chunked
//! transfer, binary integrity, and compression-plugin behavior on
//! incompressible data.

use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use rand::RngCore;

use crate::utils::{constants::MAX_BYTES_RESPONSE_SIZE, error_response::format_error_response};

/// Returns `n` random bytes as the response body.
///
/// Response headers: `Content-Type: application/octet-stream`, `Content-Length: n`.
/// `n` is capped at `MAX_BYTES_RESPONSE_SIZE` (10 MiB); larger values return 400.
/// `n = 0` returns an empty 200 OK.
///
/// The body is filled via `rand::thread_rng().fill_bytes(&mut buf[..])` for
/// maximum-entropy payloads (useful for verifying binary integrity through a
/// proxy, since any tampering is observable).
#[utoipa::path(
    get,
    path = "/bytes/{n}",
    params(
        ("n" = usize, Path, description = "Number of random bytes to return (max 10485760)")
    ),
    responses(
        (status = 200, description = "Returns n random bytes as application/octet-stream", body = Vec<u8>),
        (status = 400, description = "n exceeds MAX_BYTES_RESPONSE_SIZE")
    )
)]
pub async fn bytes_handler(axum::extract::Path(n): axum::extract::Path<usize>) -> Response {
    if n > MAX_BYTES_RESPONSE_SIZE {
        return format_error_response(
            StatusCode::BAD_REQUEST,
            &format!("Requested {n} bytes exceeds maximum of {MAX_BYTES_RESPONSE_SIZE} bytes"),
        );
    }

    let mut buf = vec![0u8; n];
    rand::thread_rng().fill_bytes(&mut buf);

    ([(header::CONTENT_TYPE, "application/octet-stream")], buf).into_response()
}

/// Creates and returns the Axum router for the bytes endpoint.
pub fn router() -> Router {
    Router::new().route("/bytes/:n", get(bytes_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_returns_n_bytes() {
        let app = router();
        let response = app
            .oneshot(Request::get("/bytes/1024").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/octet-stream"
        );

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body.len(), 1024);
    }

    #[tokio::test]
    async fn test_zero_bytes() {
        let app = router();
        let response = app
            .oneshot(Request::get("/bytes/0").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body.len(), 0);
    }

    #[tokio::test]
    async fn test_exceeds_max_returns_400() {
        let app = router();
        let response = app
            .oneshot(
                Request::get(format!("/bytes/{}", MAX_BYTES_RESPONSE_SIZE + 1))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_at_max_succeeds() {
        let app = router();
        let response = app
            .oneshot(
                Request::get(format!("/bytes/{MAX_BYTES_RESPONSE_SIZE}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body.len(), MAX_BYTES_RESPONSE_SIZE);
    }

    #[tokio::test]
    async fn test_non_numeric_path_returns_400() {
        let app = router();
        let response = app
            .oneshot(Request::get("/bytes/abc").body(Body::empty()).unwrap())
            .await
            .unwrap();

        // Axum's Path<usize> extraction failure returns 400.
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
