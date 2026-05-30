//! Graceful shutdown handling.

use axum_server::Handle;
use std::time::Duration;
use tokio::signal;

/// Grace period for in-flight requests to complete before forced shutdown.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);

/// Listens for a shutdown signal and initiates graceful shutdown.
///
/// Resolves when either **SIGINT** (Ctrl+C) or, on Unix, **SIGTERM** is
/// received, then triggers graceful shutdown on the provided `Handle` with a
/// 5-second timeout for in-flight requests.
///
/// SIGTERM handling matters because container runtimes (Docker, Kubernetes,
/// Kong Mesh / Kuma sidecars) stop a process by sending SIGTERM, *not* SIGINT.
/// Without it, the default SIGTERM disposition hard-kills the process and drops
/// in-flight requests instead of draining them. On non-Unix targets only SIGINT
/// is available, so the SIGTERM branch is compiled out.
pub async fn shutdown_signal(handle: Handle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    // No SIGTERM on non-Unix platforms; this branch never fires there.
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    let signal = tokio::select! {
        _ = ctrl_c => "SIGINT",
        _ = terminate => "SIGTERM",
    };

    tracing::info!("{signal} received, starting graceful shutdown");
    handle.graceful_shutdown(Some(SHUTDOWN_GRACE));
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    /// Sending SIGTERM must resolve `shutdown_signal` and initiate graceful
    /// shutdown — the regression this module exists to prevent (the handler
    /// previously listened for Ctrl+C/SIGINT only).
    #[tokio::test]
    async fn sigterm_triggers_graceful_shutdown() {
        let handle = Handle::new();
        let task = tokio::spawn(shutdown_signal(handle.clone()));

        // Let the spawned task be polled so the SIGTERM handler is installed
        // before we raise the signal (otherwise the default disposition would
        // terminate the whole test binary).
        tokio::time::sleep(Duration::from_millis(300)).await;

        let pid = std::process::id();
        let status = std::process::Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .status()
            .expect("failed to invoke kill");
        assert!(status.success(), "kill -TERM did not succeed");

        // shutdown_signal should return promptly once SIGTERM is delivered.
        tokio::time::timeout(Duration::from_secs(5), task)
            .await
            .expect("shutdown_signal did not return after SIGTERM")
            .expect("shutdown_signal task panicked");
    }
}
