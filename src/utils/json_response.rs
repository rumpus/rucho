// Import necessary types from Axum and Serde
use axum::{http::StatusCode, response::Response};
use serde_json::Value;

/// Formats a `serde_json::Value` into an Axum `Response`.
///
/// This function serializes the given JSON `Value` into a pretty-printed string.
/// The response will have an HTTP 200 OK status and a "Content-Type: application/json" header.
///
/// # Arguments
///
/// * `data`: A `serde_json::Value` to be serialized and sent in the response body.
///
/// # Returns
///
/// An Axum `Response` object. Returns a 500 error response if serialization fails.
pub fn format_json_response(data: Value) -> Response {
    let body = serde_json::to_string_pretty(&data);

    match body {
        Ok(json_string) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(json_string))
            .unwrap_or_else(|_| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(axum::body::Body::from(
                        r#"{"error":"Failed to build response"}"#,
                    ))
                    .expect("fallback response should always build")
            }),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                r#"{"error":"Failed to serialize response"}"#,
            ))
            .expect("fallback response should always build"),
    }
}
