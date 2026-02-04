//! Delay endpoint for testing timeout handling and slow responses.

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::any,
    Router,
};
use crate::utils::constants::MAX_DELAY_SECONDS;

/// Handles requests to the `/delay/:n` endpoint.
///
/// Introduces a delay of `n` seconds before sending a response.
/// The delay duration `n` is extracted from the path.
///
/// # Security
///
/// The maximum delay is capped at `MAX_DELAY_SECONDS` (300 seconds) to prevent
/// denial-of-service attacks where malicious requests could hold connections open indefinitely.
#[utoipa::path(
    get, post, put, patch, delete, options, head,
    path = "/delay/{n}",
    params(
        ("n" = u64, Path, description = "Number of seconds to delay the response (max 300)")
    ),
    responses(
        (status = 200, description = "Responds after the specified delay", body = String),
        (status = 400, description = "Delay exceeds maximum allowed value")
    )
)]
pub async fn delay_handler(
    axum::extract::Path(n): axum::extract::Path<u64>,
    _method: axum::http::Method,
    _body: axum::body::Body,
) -> impl IntoResponse {
    if n > MAX_DELAY_SECONDS {
        return (
            StatusCode::BAD_REQUEST,
            format!(
                "Delay of {} seconds exceeds maximum allowed value of {} seconds",
                n, MAX_DELAY_SECONDS
            ),
        )
            .into_response();
    }

    tokio::time::sleep(std::time::Duration::from_secs(n)).await;
    (StatusCode::OK, format!("Response delayed by {} seconds", n)).into_response()
}

/// Creates and returns the Axum router for the delay endpoint.
///
/// This router provides an endpoint that introduces an artificial delay before responding.
pub fn router() -> Router {
    Router::new().route("/delay/:n", any(delay_handler))
}
