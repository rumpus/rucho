//! Server setup and orchestration module.
//!
//! This module provides functionality for setting up and running the various
//! server listeners (HTTP, HTTPS, TCP, UDP) and handling graceful shutdown.

pub mod http;
pub mod metrics_layer;
pub mod shutdown;
pub mod tcp;
pub mod timing_layer;
pub mod udp;

use axum::Router;
use axum_server::Handle;
use std::sync::Arc;

use crate::utils::config::Config;

/// Runs all configured server listeners.
///
/// Sets up and starts HTTP/HTTPS, TCP, and UDP listeners based on the
/// provided configuration, then waits for a shutdown signal.
pub async fn run_server(config: &Config, app: Router) {
    let handle = Handle::new();
    let shutdown = shutdown::shutdown_signal(handle.clone());

    let mut server_handles: Vec<tokio::task::JoinHandle<Result<(), std::io::Error>>> = Vec::new();

    // Setup HTTP/HTTPS listeners
    http::setup_http_listeners(config, app.clone(), handle.clone(), &mut server_handles).await;

    // Setup TCP listener
    if let Some(tcp_addr_str) = &config.server_listen_tcp {
        tcp::setup_tcp_listener(tcp_addr_str, &mut server_handles).await;
    }

    // Setup UDP listener
    if let Some(udp_addr_str) = &config.server_listen_udp {
        if let Some(socket) = udp::bind_udp_socket(udp_addr_str).await {
            let socket = Arc::new(socket);
            udp::setup_udp_listener(socket, &mut server_handles);
        }
    }

    if !server_handles.is_empty() {
        tracing::info!(
            "{} server(s)/listener(s) started. Waiting for shutdown signal...",
            server_handles.len()
        );
        shutdown.await;
        tracing::info!("Shutdown signal received, all servers and listeners are stopping.");
    } else {
        tracing::warn!("No server or listener instances were configured or able to start.");
    }
}
