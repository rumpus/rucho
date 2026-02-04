//! Graceful shutdown handling.

use axum_server::Handle;
use std::time::Duration;
use tokio::signal;

/// Listens for a Ctrl+C signal to initiate graceful shutdown.
///
/// When a signal is received, it triggers graceful shutdown on the provided
/// `Handle` with a 5-second timeout for in-flight requests.
pub async fn shutdown_signal(handle: Handle) {
    signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    tracing::info!("Signal received, starting graceful shutdown");
    handle.graceful_shutdown(Some(Duration::from_secs(5)));
}
