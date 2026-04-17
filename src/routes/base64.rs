//! Base64 decoding endpoint.
//!
//! Decodes a URL-safe base64-encoded string from the URL path and returns the
//! decoded content along with metadata (UTF-8 validity, byte length). Accepts
//! URL-safe base64 with or without padding; standard base64 is also attempted
//! as a fallback but will not tolerate `/` in the path segment.

use axum::{extract::Path, http::StatusCode, response::Response, routing::get, Extension, Router};
use base64::Engine;
use serde_json::json;

use crate::utils::{
    constants::MAX_BASE64_INPUT_BYTES, error_response::format_error_response,
    json_response::format_json_response_with_timing, timing::RequestTiming,
};

/// Handles requests to the `/base64/:encoded` endpoint.
///
/// Decodes the URL-path base64 string and returns a JSON payload with the
/// decoded content, a UTF-8 validity flag, and the decoded byte length.
///
/// # Security
///
/// Input is capped at `MAX_BASE64_INPUT_BYTES` (4096 bytes) to prevent
/// denial-of-service attacks from oversized decode operations.
///
/// # Path Parameters
///
/// - `encoded`: The base64-encoded string to decode. URL-safe alphabet is
///   preferred; padding is optional.
///
/// # Responses
///
/// - `200 OK`: JSON object with `encoded`, `decoded`, `is_utf8`, `byte_length`,
///   and `timing.duration_ms`.
/// - `400 Bad Request`: Invalid base64 input or input exceeds the size limit.
#[utoipa::path(
    get,
    path = "/base64/{encoded}",
    params(
        ("encoded" = String, Path, description = "URL-safe base64-encoded string to decode (max 4096 bytes)")
    ),
    responses(
        (status = 200, description = "Returns decoded content with metadata", body = serde_json::Value),
        (status = 400, description = "Invalid base64 input or input exceeds size limit")
    )
)]
pub async fn base64_handler(
    Path(encoded): Path<String>,
    timing: Option<Extension<RequestTiming>>,
) -> Response {
    if encoded.len() > MAX_BASE64_INPUT_BYTES {
        return format_error_response(
            StatusCode::BAD_REQUEST,
            &format!(
                "Base64 input of {} bytes exceeds maximum allowed value of {} bytes",
                encoded.len(),
                MAX_BASE64_INPUT_BYTES
            ),
        );
    }

    // Try URL-safe variants first (expected for URL paths), then standard as fallback.
    let decoded_bytes = base64::engine::general_purpose::URL_SAFE
        .decode(&encoded)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(&encoded))
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(&encoded))
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(&encoded));

    match decoded_bytes {
        Ok(bytes) => {
            let is_utf8 = std::str::from_utf8(&bytes).is_ok();
            let decoded = String::from_utf8_lossy(&bytes).into_owned();
            let byte_length = bytes.len();

            let payload = json!({
                "encoded": encoded,
                "decoded": decoded,
                "is_utf8": is_utf8,
                "byte_length": byte_length,
            });

            let duration_ms = timing.map(|t| t.elapsed_ms());
            format_json_response_with_timing(payload, duration_ms)
        }
        Err(_) => format_error_response(StatusCode::BAD_REQUEST, "Invalid base64 input"),
    }
}

/// Creates and returns the Axum router for the base64 decoding endpoint.
pub fn router() -> Router {
    Router::new().route("/base64/:encoded", get(base64_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    async fn body_json(resp: Response) -> serde_json::Value {
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    #[tokio::test]
    async fn test_decode_url_safe_with_padding() {
        // "Hello, Rucho!" -> SGVsbG8sIFJ1Y2hvIQ==
        let response = router()
            .oneshot(
                Request::get("/base64/SGVsbG8sIFJ1Y2hvIQ==")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        assert_eq!(json["encoded"], "SGVsbG8sIFJ1Y2hvIQ==");
        assert_eq!(json["decoded"], "Hello, Rucho!");
        assert_eq!(json["is_utf8"], true);
        assert_eq!(json["byte_length"], 13);
    }

    #[tokio::test]
    async fn test_decode_url_safe_without_padding() {
        let response = router()
            .oneshot(
                Request::get("/base64/SGVsbG8sIFJ1Y2hvIQ")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        assert_eq!(json["decoded"], "Hello, Rucho!");
    }

    #[tokio::test]
    async fn test_decode_non_utf8_marked_false() {
        // URL-safe encoding of [0xFF, 0xFE, 0xFD] => "__79"
        let response = router()
            .oneshot(Request::get("/base64/__79").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        assert_eq!(json["is_utf8"], false);
        assert_eq!(json["byte_length"], 3);
    }

    #[tokio::test]
    async fn test_invalid_base64_returns_400() {
        // Length-1 input is invalid for every base64 variant.
        let response = router()
            .oneshot(Request::get("/base64/a").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_oversized_input_returns_400() {
        let oversized: String = "A".repeat(MAX_BASE64_INPUT_BYTES + 1);
        let response = router()
            .oneshot(
                Request::get(format!("/base64/{}", oversized).as_str())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
