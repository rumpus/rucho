// status.rs
use axum::{routing::any, Router, extract::Path, http::{Method, StatusCode}, response::{IntoResponse, Response}};

pub fn router() -> Router {
    Router::new().route("/status/:code", any(status_handler))
}

async fn status_handler(Path(code): Path<u16>, _method: Method) -> Response {
    StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_REQUEST).into_response()
}