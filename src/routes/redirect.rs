//! Redirect endpoint for testing HTTP redirect chain handling.

use crate::utils::constants::MAX_REDIRECT_HOPS;
use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};

/// Handles requests to the `/redirect/:n` endpoint.
///
/// Returns a 302 redirect chain that decrements `n` on each hop.
/// When `n` reaches 1, redirects to `/get` as the final destination.
/// When `n` is 0, returns 200 directly (redirect complete).
///
/// # Security
///
/// The maximum number of hops is capped at `MAX_REDIRECT_HOPS` (20) to prevent
/// abuse through excessively long redirect chains.
#[utoipa::path(
    get, post, put, patch, delete, options, head,
    path = "/redirect/{n}",
    params(
        ("n" = u32, Path, description = "Number of redirects remaining (max 20)")
    ),
    responses(
        (status = 302, description = "Redirects to /redirect/{n-1} or /get when n=1"),
        (status = 200, description = "Redirect complete (when n=0)", body = String),
        (status = 400, description = "Redirect count exceeds maximum allowed value")
    )
)]
pub async fn redirect_handler(axum::extract::Path(n): axum::extract::Path<u32>) -> Response {
    if n > MAX_REDIRECT_HOPS {
        return (
            StatusCode::BAD_REQUEST,
            format!(
                "Redirect count of {} exceeds maximum allowed value of {}",
                n, MAX_REDIRECT_HOPS
            ),
        )
            .into_response();
    }

    if n == 0 {
        return (StatusCode::OK, "Redirect complete".to_string()).into_response();
    }

    let location = if n == 1 {
        "/get".to_string()
    } else {
        format!("/redirect/{}", n - 1)
    };

    (StatusCode::FOUND, [(header::LOCATION, location)]).into_response()
}

/// Creates and returns the Axum router for the redirect endpoint.
///
/// This router provides an endpoint that returns a chain of HTTP 302 redirects.
pub fn router() -> Router {
    Router::new().route("/redirect/:n", any(redirect_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_redirect_decrements() {
        let app = router();
        let response = app
            .oneshot(Request::get("/redirect/3").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "/redirect/2"
        );
    }

    #[tokio::test]
    async fn test_redirect_one_goes_to_get() {
        let app = router();
        let response = app
            .oneshot(Request::get("/redirect/1").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/get");
    }

    #[tokio::test]
    async fn test_redirect_zero_returns_ok() {
        let app = router();
        let response = app
            .oneshot(Request::get("/redirect/0").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_redirect_exceeds_max() {
        let app = router();
        let response = app
            .oneshot(Request::get("/redirect/25").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_redirect_at_max_is_ok() {
        let app = router();
        let response = app
            .oneshot(Request::get("/redirect/20").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
    }

    #[tokio::test]
    async fn test_redirect_post_method() {
        let app = router();
        let response = app
            .oneshot(Request::post("/redirect/2").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "/redirect/1"
        );
    }
}
