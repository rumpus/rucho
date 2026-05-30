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

/// Static (non-parameterized) routes that each get their own metrics bucket.
/// Anything not listed here and not matched by a parameterized arm in
/// [`normalize_path`] collapses to `/other`, so arbitrary or unmatched paths
/// (404s from a crawler/fuzzer, Swagger assets, etc.) can't grow the metrics
/// map without bound. Add new *static* endpoints here.
const KNOWN_STATIC_PATHS: &[&str] = &[
    "/",
    "/get",
    "/post",
    "/put",
    "/patch",
    "/delete",
    "/options",
    "/healthz",
    "/endpoints",
    "/uuid",
    "/ip",
    "/user-agent",
    "/headers",
    "/anything",
    "/cookies",
    "/response-headers",
    "/xml",
    "/html",
    "/drip",
    "/gzip",
    "/deflate",
    "/brotli",
    "/metrics",
];

/// Normalizes a path for metrics collection by collapsing path parameters and
/// bucketing unknown paths.
///
/// Examples:
/// - `/status/404` -> `/status/:code`
/// - `/delay/5` -> `/delay/:n`
/// - `/anything/foo/bar` -> `/anything/*path`
/// - `/cookies/whatever` -> `/cookies/other`
/// - `/totally/unknown` -> `/other` (bounds cardinality)
fn normalize_path(path: &str) -> Cow<'static, str> {
    let segments: Vec<&str> = path.split('/').collect();

    // Parameterized routes collapse to their registered pattern.
    if segments.len() >= 3 {
        match segments.get(1) {
            Some(&"status") => return Cow::Borrowed("/status/:code"),
            Some(&"delay") => return Cow::Borrowed("/delay/:n"),
            Some(&"redirect") => return Cow::Borrowed("/redirect/:n"),
            Some(&"bytes") => return Cow::Borrowed("/bytes/:n"),
            Some(&"base64") => return Cow::Borrowed("/base64/:encoded"),
            Some(&"image") => return Cow::Borrowed("/image/:format"),
            Some(&"range") => return Cow::Borrowed("/range/:n"),
            Some(&"anything") => return Cow::Borrowed("/anything/*path"),
            Some(&"cookies") => {
                // Only set/delete are real sub-routes; bucket anything else.
                return match segments.get(2) {
                    Some(&"set") => Cow::Borrowed("/cookies/set"),
                    Some(&"delete") => Cow::Borrowed("/cookies/delete"),
                    _ => Cow::Borrowed("/cookies/other"),
                };
            }
            _ => {}
        }
    }

    // A known static route keeps its own bucket; everything else collapses to
    // `/other` to bound the number of distinct metric keys. Returning the
    // `&'static` match (not `path.to_owned()`) keeps every arm zero-alloc.
    match KNOWN_STATIC_PATHS.iter().find(|&&known| known == path) {
        Some(&known) => Cow::Borrowed(known),
        None => Cow::Borrowed("/other"),
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
    fn test_normalize_bytes_path() {
        assert_eq!(normalize_path("/bytes/1024"), "/bytes/:n");
        assert_eq!(normalize_path("/bytes/0"), "/bytes/:n");
        assert_eq!(normalize_path("/bytes/10485760"), "/bytes/:n");
    }

    #[test]
    fn test_normalize_image_path() {
        assert_eq!(normalize_path("/image/png"), "/image/:format");
        assert_eq!(normalize_path("/image/webp"), "/image/:format");
        assert_eq!(normalize_path("/image/gif"), "/image/:format");
    }

    #[test]
    fn test_normalize_range_path() {
        assert_eq!(normalize_path("/range/1024"), "/range/:n");
        assert_eq!(normalize_path("/range/0"), "/range/:n");
    }

    #[test]
    fn test_normalize_cookies_path() {
        assert_eq!(normalize_path("/cookies"), "/cookies");
        assert_eq!(normalize_path("/cookies/set"), "/cookies/set");
        assert_eq!(normalize_path("/cookies/delete"), "/cookies/delete");
    }

    #[test]
    fn test_normalize_cookies_unknown_action_buckets() {
        assert_eq!(normalize_path("/cookies/foo"), "/cookies/other");
        assert_eq!(normalize_path("/cookies/12345"), "/cookies/other");
    }

    #[test]
    fn test_normalize_base64_path() {
        assert_eq!(normalize_path("/base64/SGVsbG8="), "/base64/:encoded");
        assert_eq!(normalize_path("/base64/anything"), "/base64/:encoded");
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

    #[test]
    fn test_normalize_unknown_paths_bucket_to_other() {
        // Arbitrary/unmatched paths must not each become a distinct metric key.
        assert_eq!(normalize_path("/random123"), "/other");
        assert_eq!(normalize_path("/foo/bar"), "/other");
        assert_eq!(normalize_path("/swagger-ui/index.html"), "/other");
    }
}
