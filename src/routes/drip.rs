//! Slow drip endpoint — streams bytes evenly over a duration.
//!
//! Emits `numbytes` bytes (each `*`) over `duration` seconds, useful for
//! exercising upstream behavior in a gateway: inter-byte read/send timeouts,
//! chunked transfer encoding, and response buffering vs streaming.
//!
//! Distinct from `/delay/:n`, which exercises *first-byte* (idle) timeouts;
//! `/drip` exercises the *streaming* timeouts that fire when bytes are
//! arriving but slowly.
//!
//! Query parameters (all optional):
//! - `numbytes` — total bytes to emit (default 10, max [`MAX_DRIP_NUMBYTES`])
//! - `duration` — total stream duration in seconds (default 2, max [`MAX_DELAY_SECONDS`])
//! - `code`     — HTTP status code on the response (default 200)
//! - `delay`    — initial delay before the first byte (default 0, max [`MAX_DELAY_SECONDS`])

use axum::{
    body::Body,
    extract::Query,
    http::{header, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use futures_util::stream::{self, Stream};
use serde::Deserialize;
use std::time::Duration;

use crate::utils::{
    constants::{MAX_DELAY_SECONDS, MAX_DRIP_NUMBYTES},
    error_response::format_error_response,
};

/// Query parameters for `/drip`. All fields default if missing.
#[derive(Debug, Deserialize)]
pub struct DripParams {
    #[serde(default = "default_numbytes")]
    numbytes: usize,
    #[serde(default = "default_duration")]
    duration: u64,
    #[serde(default = "default_code")]
    code: u16,
    #[serde(default)]
    delay: u64,
}

fn default_numbytes() -> usize {
    10
}
fn default_duration() -> u64 {
    2
}
fn default_code() -> u16 {
    200
}

/// Streams `numbytes` bytes of `*` over `duration` seconds.
#[utoipa::path(
    get,
    path = "/drip",
    params(
        ("numbytes" = Option<usize>, Query, description = "Total bytes to emit (default 10, max 10000)"),
        ("duration" = Option<u64>, Query, description = "Total stream duration in seconds (default 2, max 300)"),
        ("code" = Option<u16>, Query, description = "Status code on the response (default 200)"),
        ("delay" = Option<u64>, Query, description = "Initial delay before first byte in seconds (default 0, max 300)")
    ),
    responses(
        (status = 200, description = "Bytes streamed slowly", body = Vec<u8>, content_type = "application/octet-stream"),
        (status = 400, description = "Parameter exceeds cap or invalid status code")
    )
)]
pub async fn drip_handler(Query(params): Query<DripParams>) -> Response {
    if params.numbytes > MAX_DRIP_NUMBYTES {
        return format_error_response(
            StatusCode::BAD_REQUEST,
            &format!(
                "numbytes={} exceeds maximum of {}",
                params.numbytes, MAX_DRIP_NUMBYTES
            ),
        );
    }
    if params.duration > MAX_DELAY_SECONDS {
        return format_error_response(
            StatusCode::BAD_REQUEST,
            &format!(
                "duration={} seconds exceeds maximum of {} seconds",
                params.duration, MAX_DELAY_SECONDS
            ),
        );
    }
    if params.delay > MAX_DELAY_SECONDS {
        return format_error_response(
            StatusCode::BAD_REQUEST,
            &format!(
                "delay={} seconds exceeds maximum of {} seconds",
                params.delay, MAX_DELAY_SECONDS
            ),
        );
    }
    let status = match StatusCode::from_u16(params.code) {
        Ok(s) => s,
        Err(_) => {
            return format_error_response(
                StatusCode::BAD_REQUEST,
                &format!("code={} is not a valid HTTP status code", params.code),
            );
        }
    };

    let body = Body::from_stream(build_drip_stream(
        params.numbytes,
        params.duration.saturating_mul(1000),
        Duration::from_secs(params.delay),
    ));

    match Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(body)
    {
        Ok(resp) => resp,
        Err(_) => format_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to build drip response",
        ),
    }
}

/// Builds the drip stream: emits `numbytes` bytes spread evenly over
/// `total_ms`, preceded by `initial_delay`.
///
/// Chunks are sized so that no two emissions are scheduled less than ~1 ms
/// apart, since tokio's timer precision is ~1 ms. For example, asking for
/// 10 000 bytes over 1 second yields 1 000 chunks of 10 bytes spaced 1 ms
/// apart, rather than 10 000 sub-millisecond sleeps that the timer would
/// silently coalesce.
fn build_drip_stream(
    numbytes: usize,
    total_ms: u64,
    initial_delay: Duration,
) -> impl Stream<Item = Result<Vec<u8>, std::io::Error>> {
    // For `numbytes == 0` everything is zero — no chunks, no sleeps. Otherwise
    // pick the chunk count so emissions are at least ~1 ms apart, then divide
    // unconditionally; `chunks >= 1` is guaranteed by `.max(1)`.
    let (num_chunks, interval, base, remainder) = if numbytes == 0 {
        (0usize, Duration::ZERO, 0usize, 0usize)
    } else {
        let chunks = total_ms.min(numbytes as u64).max(1) as usize;
        let interval = Duration::from_millis(total_ms / chunks as u64);
        let base = numbytes / chunks;
        let rem = numbytes % chunks;
        (chunks, interval, base, rem)
    };

    stream::unfold(0usize, move |i| async move {
        // Sleep order: initial_delay before chunk 0, then interval before each
        // subsequent chunk *and* one final interval before EOF, so the total
        // wall-clock time from request to end-of-stream equals `total_ms`.
        if i == 0 {
            if !initial_delay.is_zero() {
                tokio::time::sleep(initial_delay).await;
            }
        } else if !interval.is_zero() {
            tokio::time::sleep(interval).await;
        }
        if i >= num_chunks {
            return None;
        }
        let size = if i < remainder { base + 1 } else { base };
        let chunk = vec![b'*'; size];
        Some((Ok::<_, std::io::Error>(chunk), i + 1))
    })
}

/// Creates and returns the Axum router for the drip endpoint.
pub fn router() -> Router {
    Router::new().route("/drip", get(drip_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::Request;
    use tower::ServiceExt;

    async fn drip(query: &str) -> Response {
        router()
            .oneshot(
                Request::get(format!("/drip?{query}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn defaults_emit_10_bytes_of_stars() {
        let resp = drip("duration=0").await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/octet-stream"
        );
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body.len(), 10);
        assert!(body.iter().all(|&b| b == b'*'));
    }

    #[tokio::test]
    async fn custom_numbytes_emits_correct_count() {
        let resp = drip("numbytes=50&duration=0").await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body.len(), 50);
    }

    #[tokio::test]
    async fn custom_status_code_is_returned() {
        let resp = drip("numbytes=5&duration=0&code=504").await;
        assert_eq!(resp.status(), StatusCode::GATEWAY_TIMEOUT);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body.len(), 5);
    }

    #[tokio::test]
    async fn numbytes_over_cap_returns_400() {
        let resp = drip(&format!("numbytes={}", MAX_DRIP_NUMBYTES + 1)).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn duration_over_cap_returns_400() {
        let resp = drip(&format!("duration={}", MAX_DELAY_SECONDS + 1)).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn delay_over_cap_returns_400() {
        let resp = drip(&format!("delay={}", MAX_DELAY_SECONDS + 1)).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn invalid_status_code_returns_400() {
        // `StatusCode::from_u16` accepts 100..1000, so 1000 is the smallest
        // out-of-range value that fits in `u16`.
        let resp = drip("code=1000").await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn zero_numbytes_returns_empty_body() {
        let resp = drip("numbytes=0&duration=0").await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body.len(), 0);
    }

    #[tokio::test]
    async fn at_max_numbytes_succeeds() {
        let resp = drip(&format!("numbytes={MAX_DRIP_NUMBYTES}&duration=0")).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body.len(), MAX_DRIP_NUMBYTES);
    }

    #[tokio::test]
    async fn duration_is_respected() {
        let start = std::time::Instant::now();
        let resp = drip("numbytes=4&duration=1").await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let elapsed = start.elapsed();
        assert_eq!(body.len(), 4);
        // Total time should be ≈ duration (1 s). Allow 50 ms slack on the floor
        // to account for sub-ms remainder when num_chunks doesn't divide evenly.
        assert!(
            elapsed >= Duration::from_millis(950),
            "expected >= 950ms, got {elapsed:?}"
        );
    }
}
