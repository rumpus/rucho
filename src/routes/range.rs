//! Range endpoint — returns `n` bytes with HTTP range-request support.
//!
//! `/range/:n` serves `n` bytes of deterministic content (byte `i` is
//! `b'a' + (i % 26)`, so any slice is independently verifiable) and honors the
//! `Range` request header. A controllable upstream for testing how a gateway
//! proxies partial-content / resumable-download requests: `Accept-Ranges`,
//! `206 Partial Content` with `Content-Range`, and `416 Range Not Satisfiable`.
//!
//! Only a single range is supported (the first, if a comma-separated list is
//! sent) — multipart/byteranges responses are out of scope for a test target.

use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

use crate::utils::{constants::MAX_BYTES_RESPONSE_SIZE, error_response::format_error_response};

/// Generates the deterministic content for byte positions `[start, end)`.
///
/// Byte `i` is `b'a' + (i % 26)`, i.e. a repeating `a`..`z` pattern, so a client
/// can verify the bytes of any requested range without fetching the whole body.
fn gen_bytes(start: usize, end: usize) -> Vec<u8> {
    (start..end).map(|i| b'a' + (i % 26) as u8).collect()
}

/// Parses a `Range` header value against a resource of length `n`.
///
/// Returns the satisfiable inclusive byte range `(start, end)`, or `None` if the
/// header is malformed or unsatisfiable (caller should respond `416`). Supports
/// `bytes=start-end`, `bytes=start-` (to end), and `bytes=-suffix` (last bytes).
/// An end beyond the resource is clamped to `n - 1`.
fn parse_range(header: &str, n: usize) -> Option<(usize, usize)> {
    let spec = header.strip_prefix("bytes=")?;
    // Only the first range is honored if multiple are supplied.
    let first = spec.split(',').next()?.trim();
    let (start_s, end_s) = first.split_once('-')?;
    let (start_s, end_s) = (start_s.trim(), end_s.trim());

    if start_s.is_empty() {
        // Suffix range: "-N" → last N bytes.
        let suffix: usize = end_s.parse().ok()?;
        if suffix == 0 || n == 0 {
            return None;
        }
        let len = suffix.min(n);
        return Some((n - len, n - 1));
    }

    let start: usize = start_s.parse().ok()?;
    if start >= n {
        return None; // Start past the end → unsatisfiable.
    }
    let end = if end_s.is_empty() {
        n - 1
    } else {
        end_s.parse::<usize>().ok()?.min(n - 1)
    };
    if start > end {
        return None;
    }
    Some((start, end))
}

/// Returns `n` bytes of deterministic content with range-request support.
///
/// Without a `Range` header: `200 OK`, full body, `Accept-Ranges: bytes`.
/// With a satisfiable `Range`: `206 Partial Content` + `Content-Range`.
/// With an unsatisfiable `Range`: `416 Range Not Satisfiable` + `Content-Range: bytes */n`.
/// `n` is capped at `MAX_BYTES_RESPONSE_SIZE` (10 MiB); larger values return 400.
#[utoipa::path(
    get,
    path = "/range/{n}",
    params(
        ("n" = usize, Path, description = "Total number of bytes the resource represents (max 10485760)")
    ),
    responses(
        (status = 200, description = "Full body (no Range header)", body = Vec<u8>),
        (status = 206, description = "Partial body for a satisfiable Range", body = Vec<u8>),
        (status = 400, description = "n exceeds MAX_BYTES_RESPONSE_SIZE"),
        (status = 416, description = "Range not satisfiable")
    )
)]
pub async fn range_handler(Path(n): Path<usize>, headers: HeaderMap) -> Response {
    if n > MAX_BYTES_RESPONSE_SIZE {
        return format_error_response(
            StatusCode::BAD_REQUEST,
            &format!("Requested {n} bytes exceeds maximum of {MAX_BYTES_RESPONSE_SIZE} bytes"),
        );
    }

    let range = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);

    match range {
        None => (
            [
                (header::CONTENT_TYPE, "application/octet-stream".to_string()),
                (header::ACCEPT_RANGES, "bytes".to_string()),
            ],
            gen_bytes(0, n),
        )
            .into_response(),
        Some(r) => match parse_range(&r, n) {
            Some((start, end)) => (
                StatusCode::PARTIAL_CONTENT,
                [
                    (header::CONTENT_TYPE, "application/octet-stream".to_string()),
                    (header::ACCEPT_RANGES, "bytes".to_string()),
                    (header::CONTENT_RANGE, format!("bytes {start}-{end}/{n}")),
                ],
                gen_bytes(start, end + 1),
            )
                .into_response(),
            None => (
                StatusCode::RANGE_NOT_SATISFIABLE,
                [(header::CONTENT_RANGE, format!("bytes */{n}"))],
            )
                .into_response(),
        },
    }
}

/// Creates and returns the Axum router for the range endpoint.
pub fn router() -> Router {
    Router::new().route("/range/:n", get(range_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    // --- parse_range unit coverage ---

    #[test]
    fn parse_closed_range() {
        assert_eq!(parse_range("bytes=0-9", 100), Some((0, 9)));
        assert_eq!(parse_range("bytes=10-19", 100), Some((10, 19)));
    }

    #[test]
    fn parse_open_ended_range() {
        assert_eq!(parse_range("bytes=90-", 100), Some((90, 99)));
    }

    #[test]
    fn parse_suffix_range() {
        assert_eq!(parse_range("bytes=-10", 100), Some((90, 99)));
        // Suffix larger than the resource clamps to the whole thing.
        assert_eq!(parse_range("bytes=-500", 100), Some((0, 99)));
    }

    #[test]
    fn parse_clamps_end_past_resource() {
        assert_eq!(parse_range("bytes=50-999", 100), Some((50, 99)));
    }

    #[test]
    fn parse_rejects_unsatisfiable_and_malformed() {
        assert_eq!(parse_range("bytes=100-200", 100), None); // start past end
        assert_eq!(parse_range("bytes=-0", 100), None); // zero-length suffix
        assert_eq!(parse_range("bytes=20-10", 100), None); // start > end
        assert_eq!(parse_range("items=0-9", 100), None); // wrong unit
        assert_eq!(parse_range("bytes=abc-9", 100), None); // non-numeric
        assert_eq!(parse_range("bytes=0-9", 0), None); // empty resource
    }

    #[test]
    fn parse_takes_first_of_multiple_ranges() {
        assert_eq!(parse_range("bytes=0-9,20-29", 100), Some((0, 9)));
    }

    // --- handler behavior ---

    async fn get(path: &str, range: Option<&str>) -> Response {
        let mut req = Request::get(path);
        if let Some(r) = range {
            req = req.header(header::RANGE, r);
        }
        router()
            .oneshot(req.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn full_body_when_no_range() {
        let resp = get("/range/26", None).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get(header::ACCEPT_RANGES).unwrap(), "bytes");
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"abcdefghijklmnopqrstuvwxyz");
    }

    #[tokio::test]
    async fn partial_content_for_satisfiable_range() {
        let resp = get("/range/26", Some("bytes=0-4")).await;
        assert_eq!(resp.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(
            resp.headers().get(header::CONTENT_RANGE).unwrap(),
            "bytes 0-4/26"
        );
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"abcde");
    }

    #[tokio::test]
    async fn suffix_range_returns_tail() {
        let resp = get("/range/26", Some("bytes=-3")).await;
        assert_eq!(resp.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(
            resp.headers().get(header::CONTENT_RANGE).unwrap(),
            "bytes 23-25/26"
        );
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"xyz");
    }

    #[tokio::test]
    async fn unsatisfiable_range_returns_416() {
        let resp = get("/range/26", Some("bytes=100-200")).await;
        assert_eq!(resp.status(), StatusCode::RANGE_NOT_SATISFIABLE);
        assert_eq!(
            resp.headers().get(header::CONTENT_RANGE).unwrap(),
            "bytes */26"
        );
    }

    #[tokio::test]
    async fn exceeding_max_returns_400() {
        let resp = get(&format!("/range/{}", MAX_BYTES_RESPONSE_SIZE + 1), None).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
