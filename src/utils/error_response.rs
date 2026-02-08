// Utility to create standardized JSON error responses

use axum::{http::StatusCode, response::Response};
use serde_json::json;

/// Formats a JSON error response.
///
/// Creates a standardized JSON response object with an "error" field containing the provided message.
/// The HTTP status code and "Content-Type: application/json" header are also set.
///
/// # Arguments
///
/// * `status`: The `StatusCode` for the HTTP response.
/// * `message`: A string slice (`&str`) containing the error message.
///
/// # Returns
///
/// An Axum `Response` object. Falls back to a plain text error if JSON serialization fails.
pub fn format_error_response(status: StatusCode, message: &str) -> Response {
    let error_body = json!({
        "error": message
    });

    let body_bytes = serde_json::to_vec(&error_body).unwrap_or_else(|_| {
        format!(r#"{{"error":"{}"}}"#, message.replace('"', "\\\"")).into_bytes()
    });

    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(body_bytes))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(axum::body::Body::from(
                    r#"{"error":"Failed to build error response"}"#,
                ))
                .expect("fallback response should always build")
        })
}
