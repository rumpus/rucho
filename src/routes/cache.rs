//! Cache / conditional-request endpoints: `/cache` and `/cache/:n`.
//!
//! `/cache` returns `304 Not Modified` when the request carries a conditional
//! header (`If-None-Match` / `If-Modified-Since`), and otherwise a `200` with
//! `ETag` + `Last-Modified` so a client can revalidate next time. `/cache/:n`
//! returns `200` with `Cache-Control: public, max-age=n`.
//!
//! A controllable upstream for testing how a gateway proxies a *revalidating*
//! upstream — Kong's `proxy-cache` plugin does not itself model 304/conditional
//! revalidation, so that behavior has to originate here.

use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde_json::json;

use crate::routes::core_routes::serialize_headers;
use crate::utils::json_response::format_json_response;

/// Stable strong ETag for the cacheable resource (fixed so revalidation is
/// deterministic).
const CACHE_ETAG: &str = "\"rucho-cache-v1\"";
/// Stable `Last-Modified` date (fixed, for deterministic conditional requests).
const CACHE_LAST_MODIFIED: &str = "Mon, 01 Jan 2024 00:00:00 GMT";

/// Builds the JSON echo body shared by the cache endpoints.
fn cache_body(headers: &HeaderMap) -> serde_json::Value {
    json!({
        "method": "GET",
        "headers": serialize_headers(headers),
    })
}

/// `/cache` — conditional-request endpoint.
///
/// Returns `304 Not Modified` if the request carries `If-None-Match` or
/// `If-Modified-Since`; otherwise `200` with `ETag` + `Last-Modified` headers
/// and a JSON echo, so a client can revalidate on the next request.
#[utoipa::path(
    get,
    path = "/cache",
    responses(
        (status = 200, description = "Cacheable JSON echo with ETag + Last-Modified"),
        (status = 304, description = "Not Modified — conditional request matched")
    )
)]
pub async fn cache_handler(headers: HeaderMap) -> Response {
    if headers.contains_key(header::IF_NONE_MATCH)
        || headers.contains_key(header::IF_MODIFIED_SINCE)
    {
        return StatusCode::NOT_MODIFIED.into_response();
    }

    let mut response = format_json_response(cache_body(&headers));
    let h = response.headers_mut();
    h.insert(
        header::ETAG,
        CACHE_ETAG.parse().expect("infallible: static ETag value"),
    );
    h.insert(
        header::LAST_MODIFIED,
        CACHE_LAST_MODIFIED
            .parse()
            .expect("infallible: static Last-Modified value"),
    );
    response
}

/// `/cache/:n` — returns `200` with `Cache-Control: public, max-age=n`.
#[utoipa::path(
    get,
    path = "/cache/{n}",
    params(("n" = u64, Path, description = "max-age in seconds")),
    responses((status = 200, description = "JSON echo with Cache-Control: public, max-age=n"))
)]
pub async fn cache_seconds_handler(Path(n): Path<u64>, headers: HeaderMap) -> Response {
    let mut response = format_json_response(cache_body(&headers));
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        format!("public, max-age={n}")
            .parse()
            .expect("infallible: cache-control header value"),
    );
    response
}

/// Creates and returns the Axum router for the cache endpoints.
pub fn router() -> Router {
    Router::new()
        .route("/cache", get(cache_handler))
        .route("/cache/:n", get(cache_seconds_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_cache_sets_validators_when_unconditional() {
        let app = router();
        let resp = app
            .oneshot(Request::get("/cache").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get(header::ETAG).unwrap(), CACHE_ETAG);
        assert!(resp.headers().get(header::LAST_MODIFIED).is_some());
    }

    #[tokio::test]
    async fn test_cache_if_none_match_returns_304() {
        let app = router();
        let resp = app
            .oneshot(
                Request::get("/cache")
                    .header(header::IF_NONE_MATCH, CACHE_ETAG)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_MODIFIED);
    }

    #[tokio::test]
    async fn test_cache_if_modified_since_returns_304() {
        let app = router();
        let resp = app
            .oneshot(
                Request::get("/cache")
                    .header(header::IF_MODIFIED_SINCE, CACHE_LAST_MODIFIED)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_MODIFIED);
    }

    #[tokio::test]
    async fn test_cache_seconds_sets_cache_control() {
        let app = router();
        let resp = app
            .oneshot(Request::get("/cache/60").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CACHE_CONTROL).unwrap(),
            "public, max-age=60"
        );
    }
}
