// Import necessary types from Axum and Serde
use axum::{
    response::{Response}, // For building HTTP responses
    http::StatusCode,                   // HTTP status codes (like 200 OK)
};
use serde_json::Value;                   // Represents arbitrary JSON values

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
/// An Axum `Response` object.
pub fn format_json_response(data: Value, pretty: bool) -> Response {
    // Serialize the JSON Value to a String based on the `pretty` flag
    let body = if pretty {
        serde_json::to_string_pretty(&data).unwrap()  // Pretty-formatted (indented JSON)
    } else {
        serde_json::to_string(&data).unwrap()          // Compact (single-line JSON)
    };

    // Build and return the HTTP Response
    Response::builder()
        .status(StatusCode::OK)                        // Always returns HTTP 200 OK
        .header("Content-Type", "application/json")    // Explicit Content-Type header
        .body(axum::body::Body::from(body))             // Set serialized JSON as body
        .unwrap()                                       // Safe unwrap (controlled internal serialization)
}
