//! HTTP and HTTPS server setup.

use std::time::Duration;

use axum::Router;
use axum_server::Handle;
use hyper_util::rt::TokioTimer;
use socket2::{SockRef, TcpKeepalive};
use tokio::task::JoinHandle;

use crate::utils::config::Config;
use crate::utils::server_config;

/// Configures TCP socket options (keep-alive, nodelay) on a standard TCP listener.
///
/// Sets `SO_KEEPALIVE` with the configured idle time, probe interval, and retry count.
/// Also sets `TCP_NODELAY` based on config to disable Nagle's algorithm.
fn configure_tcp_socket(listener: &std::net::TcpListener, config: &Config) {
    let sock_ref = SockRef::from(listener);

    let keepalive = TcpKeepalive::new()
        .with_time(Duration::from_secs(config.tcp_keepalive_time))
        .with_interval(Duration::from_secs(config.tcp_keepalive_interval));

    // with_retries is not available on Windows
    #[cfg(not(target_os = "windows"))]
    let keepalive = keepalive.with_retries(config.tcp_keepalive_retries);

    if let Err(e) = sock_ref.set_tcp_keepalive(&keepalive) {
        tracing::warn!("Failed to set TCP keep-alive: {}", e);
    }

    if let Err(e) = sock_ref.set_nodelay(config.tcp_nodelay) {
        tracing::warn!("Failed to set TCP_NODELAY: {}", e);
    }
}

/// Configures HTTP-level settings on the axum_server builder.
///
/// Sets HTTP/1.1 keep-alive, header read timeout (with timer), and HTTP/2
/// keep-alive interval and timeout.
fn configure_http_builder<A>(server: &mut axum_server::Server<A>, config: &Config) {
    let http_timeout = Duration::from_secs(config.http_keep_alive_timeout);
    let header_timeout = Duration::from_secs(config.header_read_timeout);

    server
        .http_builder()
        .http1()
        .keep_alive(true)
        .timer(TokioTimer::new())
        .header_read_timeout(header_timeout);

    server
        .http_builder()
        .http2()
        .keep_alive_interval(Some(http_timeout))
        .keep_alive_timeout(Duration::from_secs(20));
}

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
            setup_http_listener(config, sock_addr, app_clone, handle_clone, server_handles).await;
        }
    }

    if server_handles.is_empty() {
        tracing::warn!("No HTTP/HTTPS server instances were configured or able to start.");
    }
}

/// Sets up an HTTP listener on the given address.
async fn setup_http_listener(
    config: &Config,
    sock_addr: std::net::SocketAddr,
    app: Router,
    handle: Handle,
    server_handles: &mut Vec<JoinHandle<Result<(), std::io::Error>>>,
) {
    match tokio::net::TcpListener::bind(sock_addr).await {
        Ok(listener) => match listener.into_std() {
            Ok(std_listener) => {
                configure_tcp_socket(&std_listener, config);

                tracing::info!("Starting HTTP server on http://{}", sock_addr);
                let mut server = axum_server::Server::from_tcp(std_listener);
                configure_http_builder(&mut server, config);
                let server_future = server.handle(handle).serve(app.into_make_service());
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
            let mut server = axum_server::bind_rustls(sock_addr, rustls_config);
            configure_http_builder(&mut server, config);
            let server_future = server.handle(handle).serve(app.into_make_service());
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
