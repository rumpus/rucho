//! Metrics collection middleware layer.
//!
//! This module provides a Tower layer that intercepts requests and responses
//! to record metrics such as request counts, endpoint hits, and status codes.

use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use std::borrow::Cow;
use std::sync::Arc;

use crate::utils::metrics::Metrics;

/// Middleware function that records request metrics.
///
/// This middleware extracts the request path and records it along with the
/// response status code to the shared metrics store.
pub async fn metrics_middleware(
    request: Request,
    next: Next,
    metrics: Arc<Metrics>,
) -> Response<Body> {
    // Normalize the path for metrics (remove path parameters).
    // Returns Cow::Borrowed for static patterns (zero alloc) or Cow::Owned for
    // passthrough/cookie paths (one alloc — down from two).
    let normalized_path = normalize_path(request.uri().path());

    // Call the inner handler
    let response = next.run(request).await;

    // Record the request with status code
    let status = response.status().as_u16();
    metrics.record_request(&normalized_path, status);

    response
}

/// Normalizes a path for metrics collection by collapsing path parameters.
///
/// Examples:
/// - `/status/404` -> `/status/:code`
/// - `/delay/5` -> `/delay/:n`
/// - `/anything/foo/bar` -> `/anything/*path`
fn normalize_path(path: &str) -> Cow<'static, str> {
    let segments: Vec<&str> = path.split('/').collect();

    if segments.len() >= 2 {
        match segments.get(1) {
            Some(&"status") if segments.len() >= 3 => Cow::Borrowed("/status/:code"),
            Some(&"delay") if segments.len() >= 3 => Cow::Borrowed("/delay/:n"),
            Some(&"redirect") if segments.len() >= 3 => Cow::Borrowed("/redirect/:n"),
            Some(&"cookies") if segments.len() >= 3 => {
                let action = segments.get(2).unwrap_or(&"");
                Cow::Owned(format!("/cookies/{action}"))
            }
            Some(&"anything") if segments.len() >= 3 => Cow::Borrowed("/anything/*path"),
            _ => Cow::Owned(path.to_owned()),
        }
    } else {
        Cow::Owned(path.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_status_path() {
        assert_eq!(normalize_path("/status/404"), "/status/:code");
        assert_eq!(normalize_path("/status/200"), "/status/:code");
        assert_eq!(normalize_path("/status/500"), "/status/:code");
    }

    #[test]
    fn test_normalize_delay_path() {
        assert_eq!(normalize_path("/delay/5"), "/delay/:n");
        assert_eq!(normalize_path("/delay/300"), "/delay/:n");
    }

    #[test]
    fn test_normalize_redirect_path() {
        assert_eq!(normalize_path("/redirect/3"), "/redirect/:n");
        assert_eq!(normalize_path("/redirect/1"), "/redirect/:n");
        assert_eq!(normalize_path("/redirect/20"), "/redirect/:n");
    }

    #[test]
    fn test_normalize_cookies_path() {
        assert_eq!(normalize_path("/cookies"), "/cookies");
        assert_eq!(normalize_path("/cookies/set"), "/cookies/set");
        assert_eq!(normalize_path("/cookies/delete"), "/cookies/delete");
    }

    #[test]
    fn test_normalize_anything_path() {
        assert_eq!(normalize_path("/anything/foo"), "/anything/*path");
        assert_eq!(normalize_path("/anything/foo/bar/baz"), "/anything/*path");
    }

    #[test]
    fn test_normalize_regular_paths() {
        assert_eq!(normalize_path("/get"), "/get");
        assert_eq!(normalize_path("/post"), "/post");
        assert_eq!(normalize_path("/healthz"), "/healthz");
        assert_eq!(normalize_path("/"), "/");
    }

    #[test]
    fn test_normalize_anything_root() {
        // /anything without additional path segments stays as is
        assert_eq!(normalize_path("/anything"), "/anything");
    }
}
