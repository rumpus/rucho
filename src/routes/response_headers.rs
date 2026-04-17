//! Response-headers endpoint.
//!
//! Mirrors each query parameter into the response headers (and JSON body) so
//! that gateway plugins operating on upstream response headers — Kong's
//! `response-transformer`, `cors`, `proxy-cache`, etc. — can be exercised
//! against a deterministic upstream.

use axum::{
    http::{HeaderName, HeaderValue, StatusCode},
    response::Response,
    routing::get,
    Extension, Router,
};
use serde_json::{Map, Value};
use std::collections::HashSet;

use crate::utils::{
    error_response::format_error_response, json_response::format_json_response_with_timing,
    timing::RequestTiming,
};

/// Echoes each query parameter as a response header and in the JSON body.
///
/// `GET /response-headers?x-rate-limit=100&cache-control=no-store` returns:
/// - Response headers `x-rate-limit: 100` and `cache-control: no-store`
/// - JSON body `{"x-rate-limit": "100", "cache-control": "no-store"}`
///
/// Duplicate keys emit multiple response headers (HTTP's repeat-header model)
/// and collapse to a JSON array in the body. Invalid header names or values
/// return 400.
///
/// User-supplied headers replace any defaults — setting `content-type`
/// overrides the default `application/json` (body remains JSON regardless;
/// the mismatch is intentional for plugin-testing scenarios).
#[utoipa::path(
    get,
    path = "/response-headers",
    responses(
        (status = 200, description = "Echoes query params as response headers and a JSON body"),
        (status = 400, description = "Invalid header name or value")
    )
)]
pub async fn response_headers_handler(
    axum::extract::Query(params): axum::extract::Query<Vec<(String, String)>>,
    timing: Option<Extension<RequestTiming>>,
) -> Response {
    // Validate and parse all headers first. On any error, short-circuit with 400
    // before mutating the response.
    let mut parsed: Vec<(HeaderName, HeaderValue)> = Vec::with_capacity(params.len());
    for (key, value) in &params {
        let header_name = match HeaderName::from_bytes(key.as_bytes()) {
            Ok(n) => n,
            Err(_) => {
                return format_error_response(
                    StatusCode::BAD_REQUEST,
                    &format!("Invalid header name: {key}"),
                );
            }
        };
        let header_value = match HeaderValue::from_str(value) {
            Ok(v) => v,
            Err(_) => {
                return format_error_response(
                    StatusCode::BAD_REQUEST,
                    &format!("Invalid header value for {key}"),
                );
            }
        };
        parsed.push((header_name, header_value));
    }

    // Build JSON body; duplicate keys collapse to arrays.
    let mut body = Map::new();
    for (key, value) in params {
        match body.remove(&key) {
            Some(Value::Array(mut arr)) => {
                arr.push(Value::String(value));
                body.insert(key, Value::Array(arr));
            }
            Some(existing) => {
                body.insert(key, Value::Array(vec![existing, Value::String(value)]));
            }
            None => {
                body.insert(key, Value::String(value));
            }
        }
    }

    let duration_ms = timing.map(|t| t.elapsed_ms());
    let mut response = format_json_response_with_timing(Value::Object(body), duration_ms);

    // User headers replace any defaults (e.g. the default `content-type:
    // application/json`). Clear defaults for every user-supplied name, then
    // append — so duplicate query keys map to duplicate response headers.
    let response_headers = response.headers_mut();
    let user_names: HashSet<HeaderName> = parsed.iter().map(|(n, _)| n.clone()).collect();
    for name in &user_names {
        response_headers.remove(name);
    }
    for (name, value) in parsed {
        response_headers.append(name, value);
    }

    response
}

/// Creates and returns the Axum router for the response-headers endpoint.
pub fn router() -> Router {
    Router::new().route("/response-headers", get(response_headers_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use serde_json::json;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_single_header() {
        let app = router();
        let response = app
            .oneshot(
                Request::get("/response-headers?x-custom=hello")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("x-custom").unwrap(), "hello");

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["x-custom"], "hello");
    }

    #[tokio::test]
    async fn test_multiple_headers() {
        let app = router();
        let response = app
            .oneshot(
                Request::get("/response-headers?x-rate-limit=100&cache-control=no-store")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.headers().get("x-rate-limit").unwrap(), "100");
        assert_eq!(response.headers().get("cache-control").unwrap(), "no-store");
    }

    #[tokio::test]
    async fn test_duplicate_keys_become_array() {
        let app = router();
        let response = app
            .oneshot(
                Request::get("/response-headers?x=1&x=2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let x_values: Vec<_> = response.headers().get_all("x").iter().collect();
        assert_eq!(x_values.len(), 2);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["x"], json!(["1", "2"]));
    }

    #[tokio::test]
    async fn test_invalid_header_name_returns_400() {
        let app = router();
        // Newline in header name is invalid per RFC 7230.
        let response = app
            .oneshot(
                Request::get("/response-headers?bad%0Aname=value")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_content_type_override() {
        let app = router();
        let response = app
            .oneshot(
                Request::get("/response-headers?content-type=text/plain")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // User-supplied content-type replaces the default application/json;
        // only one content-type header remains.
        let ct_values: Vec<_> = response.headers().get_all("content-type").iter().collect();
        assert_eq!(ct_values.len(), 1);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/plain"
        );
    }

    #[tokio::test]
    async fn test_empty_query_returns_empty_body() {
        let app = router();
        let response = app
            .oneshot(
                Request::get("/response-headers")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json, json!({}));
    }
}
