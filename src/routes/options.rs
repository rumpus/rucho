// options.rs
use axum::{routing::options, Router, response::{IntoResponse, Response}, http::{header, StatusCode}};

pub fn router() -> Router {
    Router::new().route("/options", options(options_handler))
}

async fn options_handler() -> impl IntoResponse {
    Response::builder().status(StatusCode::NO_CONTENT).header(header::ALLOW, "GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD").body(axum::body::Body::empty()).unwrap()
}