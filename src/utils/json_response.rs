// Import necessary types from Axum and Serde
use axum::{
    response::{IntoResponse, Response}, // For building HTTP responses
    http::StatusCode,                   // HTTP status codes (like 200 OK)
    Json,                                // Axum's wrapper for JSON responses (optional now)
};
use serde_json::Value;                   // Represents arbitrary JSON values

/// Takes a JSON Value and returns a properly formatted HTTP Response
/// 
/// `pretty`: controls whether the JSON is compact (default) or pretty-printed (indented).
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
