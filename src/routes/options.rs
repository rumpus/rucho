// Import necessary types
use axum::response::Response;            // Represents a full HTTP response
use axum::http::StatusCode;              // Enum for standard HTTP status codes

// Handler for the `/options` endpoint
// Responds with allowed HTTP methods and an empty body
pub async fn options_handler() -> Response {
    Response::builder()
        .status(StatusCode::NO_CONTENT)                             // 204 No Content — indicates success without a response body
        .header("Allow", "GET, POST, PUT, PATCH, DELETE, OPTIONS")  // List allowed HTTP methods
        .body(axum::body::Body::empty())                            // Empty response body
        .unwrap()                                                   // Unwrap the Result — safe here since we're building a simple, valid response
}
