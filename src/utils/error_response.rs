// Utility to create standardized JSON error responses

use axum::{
    http::StatusCode,
    response::{Response},
};
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
/// An Axum `Response` object.
pub fn format_error_response(status: StatusCode, message: &str) -> Response {
    let error_body = json!({
        "error": message
    });

    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(error_body.to_string()))
        .unwrap()
}
