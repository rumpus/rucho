// Import necessary types
use axum::{
    extract::Path,          // Extract values from URL path parameters
    http::StatusCode,       // Enum for standard HTTP status codes
    response::Response,     // Represents a full HTTP response
};

// Handler for the `/status/:code` endpoint
// Returns an empty response with the specified HTTP status code
pub async fn status_handler(Path(code): Path<u16>) -> Response {
    // Try to build a valid StatusCode from the user-provided number
    // If invalid (e.g., 999), fall back to 400 Bad Request
    let status = StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_REQUEST);

    // Build and return an empty response with the requested status code
    Response::builder()
        .status(status)
        .body(axum::body::Body::empty()) // Empty response body
        .unwrap() // Safe unwrap since building a simple static response
}