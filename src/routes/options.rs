// OPTIONS request handler for Echo Server

use axum::{
    response::{IntoResponse, Response},
    http::{header, StatusCode},
};

// Handles OPTIONS requests by returning allowed HTTP methods
pub async fn options_handler() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::NO_CONTENT)               // 204 No Content
        .header(header::ALLOW, "GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD") // Allowed methods
        .body(axum::body::Body::empty())                // No body returned
        .unwrap()
}
