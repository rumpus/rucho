// healthz.rs
use axum::{routing::get, Router, response::IntoResponse, http::StatusCode};

pub fn router() -> Router {
    Router::new().route("/healthz", get(healthz_handler))
}

async fn healthz_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}