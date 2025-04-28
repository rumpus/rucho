// Utility to create standardized JSON error responses

use axum::{
    http::StatusCode,
    response::{Response},
};
use serde_json::json;

/// Formats a JSON error response with the given status code and message
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
