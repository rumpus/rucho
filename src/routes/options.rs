use axum::response::Response;
use axum::http::StatusCode;

pub async fn options_handler() -> Response {
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("Allow", "GET, POST, PUT, PATCH, DELETE, OPTIONS")
        .body(axum::body::Body::empty())
        .unwrap()
}
