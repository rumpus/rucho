mod routes;
mod utils;

use crate::utils::config::Config;
use crate::utils::access_log::setup_access_log; // Added for access log
use clap::Parser;
use std::fs;
use std::io::Write;
use std::process;
use std::str::FromStr;
use sysinfo::{Pid, Signal, System};
use axum::Router;
// use std::net::SocketAddr; // Removed as per build error (unused import)
use std::sync::Arc;

use tokio::{net::{TcpListener, /* TcpStream, */ UdpSocket}, signal}; // TcpStream no longer needed directly here

use tower_http::{
    cors::CorsLayer,
    normalize_path::NormalizePathLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
// Updated tracing_subscriber imports for layered logging
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, filter::{self, LevelFilter}, Registry, Layer
};
use tracing_appender::non_blocking::WorkerGuard;
use axum_server::Handle;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::routes::core_routes::EndpointInfo; // Ensure EndpointInfo is imported
use crate::utils::request_models::PrettyQuery; // Ensure PrettyQuery is imported
use rucho::tcp_udp_handlers::{handle_tcp_connection, handle_udp_socket};
// Import other necessary types that are part of API responses or requests if any

// Temporarily comment out reqwest for build purposes
// use reqwest;

/// Represents the command line arguments passed to the application.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The subcommand to execute.
    #[command(subcommand)]
    command: CliCommand,
}

/// Defines the available subcommands for the CLI.
#[derive(Parser, Debug)]
pub enum CliCommand {
    /// Starts the Rucho server.
    Start {},
    /// Stops the Rucho server.
    Stop {},
    /// Checks the status of the Rucho server.
    Status {},
    /// Displays the version of Rucho.
    Version {},
}

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::core_routes::root_handler,
        routes::core_routes::get_handler,
        routes::core_routes::head_handler,
        routes::core_routes::post_handler,
        routes::core_routes::put_handler,
        routes::core_routes::patch_handler,
        routes::core_routes::delete_handler,
        routes::core_routes::options_handler,
        routes::core_routes::status_handler,
        routes::core_routes::anything_handler,
        routes::core_routes::anything_path_handler,
        routes::core_routes::endpoints_handler,
        routes::delay::delay_handler,
        routes::healthz::healthz_handler,
    ),
    components(
        schemas(EndpointInfo, PrettyQuery, routes::core_routes::Payload)
    ),
    tags(
        (name = "Rucho", description = "Rucho API")
    )
)]
struct ApiDoc;

const PID_FILE: &str = "/var/run/rucho/rucho.pid";

/// The main entry point for the Rucho application.
///
/// Parses command line arguments, initializes configuration and logging,
/// and executes the appropriate command.
#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = Config::load();

    // Setup logging
    let app_log_level = LevelFilter::from_str(&config.log_level.to_uppercase())
        .unwrap_or_else(|_| {
            eprintln!(
                "Warning: Invalid log level '{}' in config, defaulting to INFO.",
                config.log_level
            );
            LevelFilter::INFO
        });

    let (access_log_make_writer, access_log_guard) = setup_access_log(&config);

    let app_log_layer = fmt::layer()
        .with_writer(std::io::stderr) // Application logs go to stderr
        .with_filter(filter::filter_fn(|metadata| {
            // Filter for non-access logs
            !metadata.target().starts_with("tower_http::trace")
        }))
        .with_filter(app_log_level);

    let access_log_layer = fmt::layer()
        .with_writer(access_log_make_writer) // Access logs go to the configured writer
        .with_filter(filter::filter_fn(|metadata| {
            // Filter specifically for tower_http::trace events
            metadata.target().starts_with("tower_http::trace")
        }))
        .with_filter(LevelFilter::INFO); // Access logs are INFO level from TraceLayer

    Registry::default()
        .with(app_log_layer)
        .with(access_log_layer)
        .init();

    match args.command {
        CliCommand::Start {} => {
            tracing::info!("Starting server..."); // Changed from println!
            let pid = process::id();

            // Create PID directory if it doesn't exist
            let pid_dir = std::path::Path::new(PID_FILE).parent().unwrap();
            if !pid_dir.exists() {
                match fs::create_dir_all(&pid_dir) {
                    Ok(_) => {
                        tracing::info!("PID directory {} created", pid_dir.display());
                    }
                    Err(e) => {
                        tracing::error!("Failed to create PID directory {}: {}", pid_dir.display(), e);
                        // Continue to attempt PID file creation as per original logic
                    }
                }
            }

            match fs::File::create(PID_FILE) {
                Ok(mut file) => {
                    if let Err(e) = writeln!(file, "{}", pid) {
                        tracing::error!("Error: Could not write PID to {}: {}", PID_FILE, e);
                    } else {
                        tracing::info!("Server PID {} written to {}", pid, PID_FILE);
                    }
                }
                Err(e) => {
                    tracing::error!("Error: Could not create PID file {}: {}", PID_FILE, e);
                }
            }
            run_server(&config, access_log_guard).await; // Pass config and guard
        }
        CliCommand::Stop {} => {
            match fs::read_to_string(PID_FILE) {
                Ok(pid_str) => {
                    match pid_str.trim().parse::<usize>() {
                        Ok(pid_val) => {
                            let pid = Pid::from(pid_val);
                            let mut s = System::new_all(); // SysInfoSystemExt is used here
                            s.refresh_processes(); 
                            if let Some(process) = s.process(pid) {
                                tracing::info!("Stopping server (PID: {})...", pid);
                                match process.kill_with(Signal::Term) {
                                    Some(true) => {
                                        tracing::info!("Termination signal sent to process {}.", pid);
                                        std::thread::sleep(std::time::Duration::from_secs(1));
                                        s.refresh_processes();
                                        if s.process(pid).is_none() {
                                           tracing::info!("Server stopped successfully.");
                                           if let Err(e) = fs::remove_file(PID_FILE) {
                                               tracing::warn!("Warning: Could not remove PID file {}: {}", PID_FILE, e);
                                           }
                                        } else {
                                           tracing::info!("Process {} still running. You might need to use kill -9.", pid);
                                        }
                                    }
                                    Some(false) => {
                                        tracing::error!("Error: Failed to send termination signal to process {} (signal not sent or process already terminating).", pid);
                                         s.refresh_processes();
                                        if s.process(pid).is_none() {
                                            tracing::info!("Server process {} seems to have already stopped.", pid);
                                            if let Err(e) = fs::remove_file(PID_FILE) {
                                               tracing::warn!("Warning: Could not remove PID file {}: {}", PID_FILE, e);
                                           }
                                        }
                                    }
                                    None => {
                                        tracing::error!("Error: Failed to send termination signal to process {} (process may not exist or permissions issue for signalling).", pid);
                                        s.refresh_processes();
                                        if s.process(pid).is_none() {
                                            tracing::info!("Server process {} seems to have already stopped or does not exist.", pid);
                                            if let Err(e) = fs::remove_file(PID_FILE) {
                                               tracing::warn!("Warning: Could not remove PID file {}: {}", PID_FILE, e);
                                           }
                                        }
                                    }
                                }
                            } else {
                                tracing::info!("Process with PID {} not found. It might have already stopped.", pid);
                                if let Err(e) = fs::remove_file(PID_FILE) {
                                    tracing::warn!("Warning: Could not remove stale PID file {}: {}", PID_FILE, e);
                                }
                            }
                        }
                        Err(_) => tracing::error!("Error: Invalid PID format in {}.", PID_FILE),
                    }
                }
                Err(_) => tracing::info!("Server not running (PID file {} not found).", PID_FILE),
            }
        }
        CliCommand::Status {} => {
            match fs::read_to_string(PID_FILE) {
                Ok(pid_str) => {
                    match pid_str.trim().parse::<usize>() {
                        Ok(pid_val) => {
                            let pid = Pid::from(pid_val);
                            let mut s = System::new_all();
                            s.refresh_processes();
                            if let Some(_process) = s.process(pid) {
                                tracing::info!("Server is running (PID: {}).", pid);
                                tracing::info!("Health check functionality is currently disabled.");
                            } else {
                                tracing::info!("Server is stopped (PID file {} found, but process {} not running).", PID_FILE, pid);
                                tracing::info!("Consider running 'rucho stop' to attempt cleanup or manually deleting {}.", PID_FILE);
                            }
                        }
                        Err(_) => tracing::error!("Error: Invalid PID format in {}. Consider deleting it.", PID_FILE),
                    }
                }
                Err(_) => tracing::info!("Server is stopped (PID file {} not found).", PID_FILE),
            }
        }
        CliCommand::Version {} => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")); // Version can still be println!
        }
    }
}

/// Runs the Axum web server with the provided configuration.
///
/// This function sets up the HTTP/S listeners, configures routing,
/// and handles graceful shutdown.
async fn run_server(config: &Config, _access_log_guard: Option<WorkerGuard>) {
    // _access_log_guard is now passed in and kept in scope.

    let handle = Handle::new();
    let shutdown = shutdown_signal(handle.clone());

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(routes::core_routes::router())
        .merge(routes::healthz::router())
        .merge(routes::delay::router())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        .layer(CorsLayer::permissive())
        .layer(NormalizePathLayer::trim_trailing_slash());

    let mut listeners_to_start: Vec<(String, bool)> = Vec::new();

    if let Some(parsed) = crate::utils::server_config::parse_listen_address(&config.server_listen_primary) {
        listeners_to_start.push(parsed);
    }
    if let Some(parsed) = crate::utils::server_config::parse_listen_address(&config.server_listen_secondary) {
        listeners_to_start.push(parsed);
    }

    let mut server_handles: Vec<tokio::task::JoinHandle<Result<(), std::io::Error>>> = Vec::new();

    for (address_str, is_ssl) in listeners_to_start {
        let app_clone = app.clone();
        let handle_clone = handle.clone();

        let sock_addr: std::net::SocketAddr = match address_str.parse() {
            Ok(addr) => addr,
            Err(e) => {
                tracing::error!("Failed to parse address '{}': {}. Skipping this listener.", address_str, e);
                continue;
            }
        };

        if is_ssl {
            match crate::utils::server_config::try_load_rustls_config(config.ssl_cert.as_deref(), config.ssl_key.as_deref()).await {
                Some(rustls_config) => {
                    tracing::info!("Starting HTTPS server on https://{}", sock_addr);
                    let server_future = axum_server::bind_rustls(sock_addr, rustls_config)
                        .handle(handle_clone)
                        .serve(app_clone.into_make_service());
                    server_handles.push(tokio::spawn(server_future));
                }
                None => {
                    tracing::error!("Failed to load Rustls config for {}: HTTPS server not started. Check SSL certificate/key configuration and paths.", sock_addr);
                }
            }
        } else {
            match tokio::net::TcpListener::bind(sock_addr).await {
                Ok(listener) => {
                    match listener.into_std() {
                        Ok(std_listener) => {
                            tracing::info!("Starting HTTP server on http://{}", sock_addr);
                            let server_future = axum_server::Server::from_tcp(std_listener)
                                .handle(handle_clone)
                                .serve(app_clone.into_make_service());
                            server_handles.push(tokio::spawn(server_future));
                        }
                        Err(e) => {
                             tracing::error!("Failed to convert tokio listener to std for {}: {}. Skipping this listener.", sock_addr, e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to bind HTTP listener for {}: {}. Skipping this listener.", sock_addr, e);
                }
            }
        }
    }

    if !server_handles.is_empty() {
        tracing::info!("{} server(s) started. Waiting for shutdown signal...", server_handles.len());
    } else {
        tracing::warn!("No HTTP/HTTPS server instances were configured or able to start.");
    }

    // TCP listeners
    if let Some(tcp_addr_str) = &config.server_listen_tcp {
        match tcp_addr_str.parse::<std::net::SocketAddr>() {
            Ok(addr) => {
                match TcpListener::bind(addr).await {
                    Ok(listener) => {
                        tracing::info!("Starting TCP listener on {}", addr);
                        let tcp_listener_handle = tokio::spawn(async move {
                            loop {
                                match listener.accept().await {
                                    Ok((socket, client_addr)) => {
                                        tracing::info!("Accepted new TCP connection from {}", client_addr);
                                        tokio::spawn(handle_tcp_connection(socket));
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to accept TCP connection: {}. Listener loop continues.", e);
                                        // Potentially add a small delay here if accept errors are persistent
                                    }
                                }
                            }
                            // This part is unreachable for a loop that never breaks based on accept errors
                            // but is added for type compatibility if we were to add break conditions.
                            #[allow(unreachable_code)]
                            Ok::<(), std::io::Error>(())
                        });
                        server_handles.push(tcp_listener_handle);
                    }
                    Err(e) => {
                        tracing::error!("Failed to bind TCP listener for {}: {}", addr, e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to parse TCP address '{}': {}", tcp_addr_str, e);
            }
        }
    }

    // UDP listeners
    if let Some(udp_addr_str) = &config.server_listen_udp {
        match udp_addr_str.parse::<std::net::SocketAddr>() {
            Ok(addr) => {
                match UdpSocket::bind(addr).await {
                    Ok(socket) => {
                        tracing::info!("Starting UDP listener on {}", addr);
                        let socket = Arc::new(socket);
                        let udp_handle = tokio::spawn(handle_udp_socket(socket.clone()));
                        server_handles.push(udp_handle);
                    }
                    Err(e) => {
                        tracing::error!("Failed to bind UDP listener for {}: {}", addr, e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to parse UDP address '{}': {}", udp_addr_str, e);
            }
        }
    }

    if !server_handles.is_empty() {
        shutdown.await; // This is `shutdown_signal(handle.clone())` passed from main
        tracing::info!("Shutdown signal received, all servers and listeners are stopping.");
        // Note: TCP/UDP listeners started with tokio::spawn need to handle shutdown internally or will be aborted.
        // The Axum server handles have graceful shutdown. For TCP/UDP, a more robust shutdown might involve
        // sending signals to the spawned tasks or using cancellation tokens.
    } else {
        tracing::warn!("No server or listener instances were configured or able to start in total.");
    }
}

// Placeholder functions `handle_tcp_connection` and `handle_udp_socket` are removed,
// as we now use the implementations from `crate::tcp_udp_handlers`.

/// Listens for a Ctrl+C signal to initiate a graceful shutdown of the server.
async fn shutdown_signal(handle: Handle) {
    signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    tracing::info!("Signal received, starting graceful shutdown");
    handle.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
}
