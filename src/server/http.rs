//! HTTP and HTTPS server setup.

use axum::Router;
use axum_server::Handle;
use tokio::task::JoinHandle;

use crate::utils::config::Config;
use crate::utils::server_config;

/// Sets up HTTP and HTTPS listeners based on configuration.
///
/// Parses the primary and secondary listen addresses from config,
/// determines if SSL should be used, and spawns the appropriate server tasks.
pub async fn setup_http_listeners(
    config: &Config,
    app: Router,
    handle: Handle,
    server_handles: &mut Vec<JoinHandle<Result<(), std::io::Error>>>,
) {
    let mut listeners_to_start: Vec<(String, bool)> = Vec::new();

    if let Some(parsed) = server_config::parse_listen_address(&config.server_listen_primary) {
        listeners_to_start.push(parsed);
    }
    if let Some(parsed) = server_config::parse_listen_address(&config.server_listen_secondary) {
        listeners_to_start.push(parsed);
    }

    for (address_str, is_ssl) in listeners_to_start {
        let app_clone = app.clone();
        let handle_clone = handle.clone();

        let sock_addr: std::net::SocketAddr = match address_str.parse() {
            Ok(addr) => addr,
            Err(e) => {
                tracing::error!(
                    "Failed to parse address '{}': {}. Skipping this listener.",
                    address_str,
                    e
                );
                continue;
            }
        };

        if is_ssl {
            setup_https_listener(config, sock_addr, app_clone, handle_clone, server_handles).await;
        } else {
            setup_http_listener(sock_addr, app_clone, handle_clone, server_handles).await;
        }
    }

    if server_handles.is_empty() {
        tracing::warn!("No HTTP/HTTPS server instances were configured or able to start.");
    }
}

/// Sets up an HTTP listener on the given address.
async fn setup_http_listener(
    sock_addr: std::net::SocketAddr,
    app: Router,
    handle: Handle,
    server_handles: &mut Vec<JoinHandle<Result<(), std::io::Error>>>,
) {
    match tokio::net::TcpListener::bind(sock_addr).await {
        Ok(listener) => match listener.into_std() {
            Ok(std_listener) => {
                tracing::info!("Starting HTTP server on http://{}", sock_addr);
                let server_future = axum_server::Server::from_tcp(std_listener)
                    .handle(handle)
                    .serve(app.into_make_service());
                server_handles.push(tokio::spawn(server_future));
            }
            Err(e) => {
                tracing::error!(
                    "Failed to convert tokio listener to std for {}: {}. Skipping this listener.",
                    sock_addr,
                    e
                );
            }
        },
        Err(e) => {
            tracing::error!(
                "Failed to bind HTTP listener for {}: {}. Skipping this listener.",
                sock_addr,
                e
            );
        }
    }
}

/// Sets up an HTTPS listener on the given address.
async fn setup_https_listener(
    config: &Config,
    sock_addr: std::net::SocketAddr,
    app: Router,
    handle: Handle,
    server_handles: &mut Vec<JoinHandle<Result<(), std::io::Error>>>,
) {
    match server_config::try_load_rustls_config(
        config.ssl_cert.as_deref(),
        config.ssl_key.as_deref(),
    )
    .await
    {
        Some(rustls_config) => {
            tracing::info!("Starting HTTPS server on https://{}", sock_addr);
            let server_future = axum_server::bind_rustls(sock_addr, rustls_config)
                .handle(handle)
                .serve(app.into_make_service());
            server_handles.push(tokio::spawn(server_future));
        }
        None => {
            tracing::error!(
                "Failed to load Rustls config for {}: HTTPS server not started. \
                Check SSL certificate/key configuration and paths.",
                sock_addr
            );
        }
    }
}
