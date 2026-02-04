// healthz.rs
use axum::{routing::get, Router, response::IntoResponse, http::StatusCode};

/// Creates and returns the Axum router for the health check endpoint.
///
/// This router provides a simple `/healthz` endpoint that returns an HTTP 200 OK status.
pub fn router() -> Router {
    Router::new().route("/healthz", get(healthz_handler))
}

/// Handles requests to the `/healthz` endpoint.
///
/// Returns an HTTP 200 OK status and the plain text "OK".
#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Health check successful", body = String)
    )
)]
pub async fn healthz_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}