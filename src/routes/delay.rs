// delay.rs
use axum::{routing::any, Router, body::Body, extract::Path, http::Method, response::IntoResponse};
use std::time::Duration;
use tokio::time::sleep;

pub fn router() -> Router {
    Router::new().route("/delay/:n", any(delay_handler))
}

async fn delay_handler(Path(n): Path<u64>, _method: Method, _body: Body) -> impl IntoResponse {
    sleep(Duration::from_secs(n)).await;
    format!("Response delayed by {} seconds", n)
}
