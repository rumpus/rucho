// delay.rs
use axum::{routing::any, Router};
// Unused imports were: body::Body, extract::Path, http::Method, response::IntoResponse
// Unused imports from std/tokio were: std::time::Duration, tokio::time::sleep

/// Handles requests to the `/delay/:n` endpoint.
///
/// Introduces a delay of `n` seconds before sending a response.
/// The delay duration `n` is extracted from the path.
#[utoipa::path(
    get, post, put, patch, delete, options, head,
    path = "/delay/{n}",
    params(
        ("n" = u64, Path, description = "Number of seconds to delay the response")
    ),
    responses(
        (status = 200, description = "Responds after the specified delay", body = String)
    )
)]
async fn delay_handler(axum::extract::Path(n): axum::extract::Path<u64>, _method: axum::http::Method, _body: axum::body::Body) -> impl axum::response::IntoResponse {
    tokio::time::sleep(std::time::Duration::from_secs(n)).await;
    format!("Response delayed by {} seconds", n)
}

/// Creates and returns the Axum router for the delay endpoint.
///
/// This router provides an endpoint that introduces an artificial delay before responding.
pub fn router() -> Router {
    Router::new().route("/delay/:n", any(delay_handler))
}
