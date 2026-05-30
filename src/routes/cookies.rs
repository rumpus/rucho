//! Cookie inspection and manipulation endpoints.
//!
//! Provides endpoints for inspecting cookies sent with a request, setting new
//! cookies via response headers, and deleting cookies by expiring them.

use axum::{
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use serde_json::json;
use std::collections::HashMap;

use crate::utils::{json_response::format_json_response_with_timing, timing::RequestTiming};

/// Parses the `Cookie` header into a map of name-value pairs.
///
/// Splits on `;` (tolerating optional surrounding whitespace, per RFC 6265's
/// lenient parsing — so both `a=1; b=2` and `a=1;b=2` work) then on `=` to
/// extract cookie names and values. Cookies without a `=` are ignored.
fn parse_cookies(headers: &HeaderMap) -> HashMap<String, String> {
    headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .map(|cookie_str| {
            cookie_str
                .split(';')
                .filter_map(|pair| {
                    let mut parts = pair.splitn(2, '=');
                    let name = parts.next()?.trim();
                    let value = parts.next().unwrap_or("").trim();
                    if name.is_empty() {
                        None
                    } else {
                        Some((name.to_string(), value.to_string()))
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Returns all cookies from the request as JSON.
///
/// Parses the `Cookie` header and returns a JSON object with a `cookies` key
/// containing a map of cookie name-value pairs.
///
/// # Example Response
///
/// ```json
/// { "cookies": { "session": "abc123", "theme": "dark" } }
/// ```
#[utoipa::path(
    get,
    path = "/cookies",
    responses(
        (status = 200, description = "Returns all cookies from the request", body = serde_json::Value)
    )
)]
pub async fn cookies_handler(
    headers: HeaderMap,
    timing: Option<Extension<RequestTiming>>,
) -> Response {
    let cookies = parse_cookies(&headers);
    let duration_ms = timing.map(|t| t.elapsed_ms());
    format_json_response_with_timing(json!({"cookies": cookies}), duration_ms)
}

/// Reserved query-param keys that set `Set-Cookie` *attributes* rather than
/// cookies: `secure`/`httponly` (presence-only flags), `samesite`, `max_age`,
/// `path` (defaults to `/`), and `domain`. Every other param becomes a cookie.
const RESERVED_ATTRS: &[&str] = &[
    "secure", "httponly", "samesite", "max_age", "path", "domain",
];

/// Sets cookies from query parameters and redirects to `/cookies`.
///
/// Each non-reserved query parameter becomes a `Set-Cookie` response header.
/// Reserved keys add attributes applied to every cookie set in the request:
/// `secure`, `httponly`, `samesite=<Strict|Lax|None>`, `max_age=<seconds>`,
/// `path=<path>` (default `/`), `domain=<domain>`. After setting the cookies,
/// responds with a 302 redirect to `/cookies` so the client can see the result.
///
/// # Example
///
/// `GET /cookies/set?session=abc&secure&httponly&samesite=Strict&max_age=3600`
/// sets `session=abc; Path=/; Max-Age=3600; SameSite=Strict; Secure; HttpOnly`.
#[utoipa::path(
    get,
    path = "/cookies/set",
    params(
        ("" = HashMap<String, String>, Query, description = "Cookie name=value pairs, plus optional attribute flags: secure, httponly, samesite, max_age, path, domain")
    ),
    responses(
        (status = 302, description = "Redirects to /cookies after setting cookies")
    )
)]
pub async fn set_cookies_handler(
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    // Build the attribute suffix shared by every cookie set in this request.
    let path = params.get("path").map(String::as_str).unwrap_or("/");
    let mut attrs = format!("; Path={path}");
    if let Some(domain) = params.get("domain") {
        attrs.push_str(&format!("; Domain={domain}"));
    }
    if let Some(max_age) = params.get("max_age") {
        attrs.push_str(&format!("; Max-Age={max_age}"));
    }
    if let Some(samesite) = params.get("samesite") {
        attrs.push_str(&format!("; SameSite={samesite}"));
    }
    if params.contains_key("secure") {
        attrs.push_str("; Secure");
    }
    if params.contains_key("httponly") {
        attrs.push_str("; HttpOnly");
    }

    let mut response = (StatusCode::FOUND, [(header::LOCATION, "/cookies")]).into_response();
    let response_headers = response.headers_mut();

    for (name, value) in &params {
        if RESERVED_ATTRS.contains(&name.as_str()) {
            continue; // attribute, not a cookie
        }
        if let Ok(cookie_val) = header::HeaderValue::from_str(&format!("{name}={value}{attrs}")) {
            response_headers.append(header::SET_COOKIE, cookie_val);
        }
    }

    response
}

/// Deletes cookies by setting `Max-Age=0` and redirects to `/cookies`.
///
/// Each query parameter name is used to expire the corresponding cookie.
/// The value of the query parameter is ignored.
///
/// # Example
///
/// `GET /cookies/delete?foo&theme` expires both cookies and redirects.
#[utoipa::path(
    get,
    path = "/cookies/delete",
    params(
        ("" = HashMap<String, String>, Query, description = "Cookie names to delete")
    ),
    responses(
        (status = 302, description = "Redirects to /cookies after deleting cookies")
    )
)]
pub async fn delete_cookies_handler(
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let mut response = (StatusCode::FOUND, [(header::LOCATION, "/cookies")]).into_response();
    let response_headers = response.headers_mut();

    for name in params.keys() {
        if let Ok(cookie_val) =
            header::HeaderValue::from_str(&format!("{name}=; Max-Age=0; Path=/"))
        {
            response_headers.append(header::SET_COOKIE, cookie_val);
        }
    }

    response
}

/// Creates and returns the Axum router for the cookie endpoints.
///
/// Registers `/cookies`, `/cookies/set`, and `/cookies/delete`.
pub fn router() -> Router {
    Router::new()
        .route("/cookies", get(cookies_handler))
        .route("/cookies/set", get(set_cookies_handler))
        .route("/cookies/delete", get(delete_cookies_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_cookies_empty() {
        let app = router();
        let response = app
            .oneshot(Request::get("/cookies").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["cookies"], json!({}));
    }

    #[tokio::test]
    async fn test_cookies_with_values() {
        let app = router();
        let response = app
            .oneshot(
                Request::get("/cookies")
                    .header(header::COOKIE, "foo=bar; baz=qux")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["cookies"]["foo"], "bar");
        assert_eq!(json["cookies"]["baz"], "qux");
    }

    #[tokio::test]
    async fn test_set_cookies_redirects() {
        let app = router();
        let response = app
            .oneshot(
                Request::get("/cookies/set?foo=bar&theme=dark")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "/cookies"
        );

        let set_cookies: Vec<&str> = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .collect();

        assert_eq!(set_cookies.len(), 2);
        assert!(set_cookies.iter().any(|c| c.contains("foo=bar")));
        assert!(set_cookies.iter().any(|c| c.contains("theme=dark")));
    }

    #[tokio::test]
    async fn test_delete_cookies_redirects() {
        let app = router();
        let response = app
            .oneshot(
                Request::get("/cookies/delete?foo&theme")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "/cookies"
        );

        let set_cookies: Vec<&str> = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .collect();

        assert_eq!(set_cookies.len(), 2);
        assert!(set_cookies
            .iter()
            .any(|c| c.contains("foo=") && c.contains("Max-Age=0")));
        assert!(set_cookies
            .iter()
            .any(|c| c.contains("theme=") && c.contains("Max-Age=0")));
    }

    #[tokio::test]
    async fn test_set_cookies_with_path() {
        let app = router();
        let response = app
            .oneshot(
                Request::get("/cookies/set?session=abc123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let set_cookie = response
            .headers()
            .get(header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap();

        assert!(set_cookie.contains("Path=/"));
    }

    #[tokio::test]
    async fn test_set_cookies_with_attributes() {
        let app = router();
        let response = app
            .oneshot(
                Request::get(
                    "/cookies/set?session=abc&secure&httponly&samesite=Strict&max_age=3600",
                )
                .body(Body::empty())
                .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        let set_cookies: Vec<&str> = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .collect();

        // Only `session` is a cookie; the reserved keys become attributes.
        assert_eq!(set_cookies.len(), 1);
        let c = set_cookies[0];
        assert!(c.contains("session=abc"), "cookie: {c}");
        assert!(c.contains("Secure"), "cookie: {c}");
        assert!(c.contains("HttpOnly"), "cookie: {c}");
        assert!(c.contains("SameSite=Strict"), "cookie: {c}");
        assert!(c.contains("Max-Age=3600"), "cookie: {c}");
    }

    #[test]
    fn test_parse_cookies_basic() {
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, "a=1; b=2".parse().unwrap());
        let cookies = parse_cookies(&headers);
        assert_eq!(cookies.get("a").unwrap(), "1");
        assert_eq!(cookies.get("b").unwrap(), "2");
    }

    #[test]
    fn test_parse_cookies_empty_header() {
        let headers = HeaderMap::new();
        let cookies = parse_cookies(&headers);
        assert!(cookies.is_empty());
    }

    #[test]
    fn test_parse_cookies_value_with_equals() {
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, "token=abc=def=ghi".parse().unwrap());
        let cookies = parse_cookies(&headers);
        assert_eq!(cookies.get("token").unwrap(), "abc=def=ghi");
    }

    #[test]
    fn test_parse_cookies_tolerates_missing_space() {
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, "a=1;b=2".parse().unwrap());
        let cookies = parse_cookies(&headers);
        assert_eq!(cookies.get("a").unwrap(), "1");
        assert_eq!(cookies.get("b").unwrap(), "2");
    }
}
