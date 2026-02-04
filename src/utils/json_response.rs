// Import necessary types from Axum and Serde
use axum::{
    response::Response,
    http::StatusCode,
};
use serde_json::Value;

/// Formats a `serde_json::Value` into an Axum `Response`.
///
/// This function serializes the given JSON `Value` into a string.
/// The response will have an HTTP 200 OK status and a "Content-Type: application/json" header.
///
/// # Arguments
///
/// * `data`: A `serde_json::Value` to be serialized and sent in the response body.
/// * `pretty`: A boolean indicating whether the JSON output should be pretty-printed (true)
///   or compact (false).
///
/// # Returns
///
/// An Axum `Response` object. Returns a 500 error response if serialization fails.
pub fn format_json_response(data: Value, pretty: bool) -> Response {
    // Serialize the JSON Value to a String based on the `pretty` flag
    let body = if pretty {
        serde_json::to_string_pretty(&data)
    } else {
        serde_json::to_string(&data)
    };

    match body {
        Ok(json_string) => {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(json_string))
                .unwrap_or_else(|_| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(axum::body::Body::from(r#"{"error":"Failed to build response"}"#))
                        .expect("fallback response should always build")
                })
        }
        Err(_) => {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(r#"{"error":"Failed to serialize response"}"#))
                .expect("fallback response should always build")
        }
    }
}
