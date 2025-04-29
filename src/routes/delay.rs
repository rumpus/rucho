// Import necessary types
use axum::{
    extract::Path,
    response::IntoResponse,
};
use std::time::Duration;
use tokio::time::sleep;

// Handler for the `/delay/:n` endpoint
// Delays the response by `n` seconds
pub async fn delay_handler(Path(n): Path<u64>) -> impl IntoResponse {
    // Sleep for `n` seconds
    sleep(Duration::from_secs(n)).await;

    // Return a simple response after the delay
    format!("Response delayed by {} seconds", n)
}