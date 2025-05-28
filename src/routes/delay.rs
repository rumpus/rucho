// delay.rs
use axum::{routing::any, Router, body::Body, extract::Path, http::Method, response::IntoResponse};
use std::time::Duration;
use tokio::time::sleep;

/// Creates and returns the Axum router for the delay endpoint.
///
/// This router provides an endpoint that introduces an artificial delay before responding.
pub fn router() -> Router {
    Router::new().route("/delay/:n", any(delay_handler))
}

/// Handles requests to the `/delay/:n` endpoint.
///
/// Introduces a delay of `n` seconds before sending a response.
/// The delay duration `n` is extracted from the path.
#[utoipa::path(
    all, // Represents ANY method
    path = "/delay/{n}",
    params(
        ("n" = u64, Path, description = "Number of seconds to delay the response")
    ),
    responses(
        (status = 200, description = "Response delayed successfully", body = String)
    )
)]
async fn delay_handler(Path(n): Path<u64>, _method: Method, _body: Body) -> impl IntoResponse {
    sleep(Duration::from_secs(n)).await;
    format!("Response delayed by {} seconds", n)
}
