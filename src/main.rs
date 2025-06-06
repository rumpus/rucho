mod routes; // Declares the routes module, containing all API route handlers.
mod utils; // Declares the utils module, providing utility functions and structures.

use crate::utils::config::Config; // For loading application configuration.
use clap::Parser; // To parse command-line arguments.
use std::fs; // For file system operations (e.g., reading/writing PID file).
use std::io::Write; // For writing to files (e.g., PID file).
use std::process; // For getting the current process ID.
use std::str::FromStr; // For converting strings to other types (e.g., LogLevel).
use sysinfo::{Pid, Signal, System}; // For system information, used here to find and kill processes by PID.
use axum::Router; // The main router type from the Axum web framework.
// use std::net::SocketAddr; // Potentially for socket address parsing if not done by axum/tokio.
use tokio::signal; // For handling asynchronous signals (e.g., Ctrl+C for shutdown).
use tower_http::{ // Provides HTTP-specific middleware.
    cors::CorsLayer, // Middleware for Cross-Origin Resource Sharing.
    normalize_path::NormalizePathLayer, // Middleware for normalizing request paths (e.g., trimming trailing slashes).
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer}, // Middleware for tracing requests and responses.
};
use tracing::Level; // Represents logging levels (e.g., INFO, DEBUG).
use tracing_subscriber; // For initializing and configuring the tracing (logging) system.
use axum_server::Handle; // For graceful shutdown of the Axum server.
// crate::utils::server_config::try_load_rustls_config will be used directly with crate:: prefix for clarity.
use utoipa::OpenApi; // For generating OpenAPI (Swagger) specifications.
use utoipa_swagger_ui::SwaggerUi; // For serving Swagger UI for the OpenAPI spec.
use crate::routes::core_routes::EndpointInfo; // Data structure representing API endpoint information, used in OpenAPI spec.
use crate::utils::request_models::PrettyQuery; // Data structure for common query parameters (e.g., `pretty`), used in OpenAPI spec.
// Import other necessary types that are part of API responses or requests if any.

// Temporarily comment out reqwest for build purposes
// use reqwest; // HTTP client, might be used for health checks or other internal requests.

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Parser, Debug)]
pub enum CliCommand {
    Start {},
    Stop {},
    Status {},
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

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = Config::load();

    let log_level = Level::from_str(&config.log_level.to_uppercase())
        .unwrap_or_else(|_| {
            eprintln!("Warning: Invalid log level '{}' in config, defaulting to INFO.", config.log_level);
            Level::INFO
        });
    tracing_subscriber::fmt().with_max_level(log_level).init();

    match args.command {
        CliCommand::Start {} => {
            println!("Starting server...");
            let pid = process::id();
            match fs::File::create(PID_FILE) {
                Ok(mut file) => {
                    if let Err(e) = writeln!(file, "{}", pid) {
                        eprintln!("Error: Could not write PID to {}: {}", PID_FILE, e);
                    } else {
                        println!("Server PID {} written to {}", pid, PID_FILE);
                    }
                }
                Err(e) => {
                    eprintln!("Error: Could not create PID file {}: {}", PID_FILE, e);
                }
            }
            run_server(&config).await;
        }
        CliCommand::Stop {} => {
            match fs::read_to_string(PID_FILE) {
                Ok(pid_str) => {
                    match pid_str.trim().parse::<usize>() {
                        Ok(pid_val) => {
                            let pid = Pid::from(pid_val);
                            let mut s = System::new_all();
                            s.refresh_processes();
                            if let Some(process) = s.process(pid) {
                                println!("Stopping server (PID: {})...", pid);
                                match process.kill_with(Signal::Term) {
                                    Some(true) => {
                                        println!("Termination signal sent to process {}.", pid);
                                        std::thread::sleep(std::time::Duration::from_secs(1));
                                        s.refresh_processes();
                                        if s.process(pid).is_none() {
                                           println!("Server stopped successfully.");
                                           if let Err(e) = fs::remove_file(PID_FILE) {
                                               eprintln!("Warning: Could not remove PID file {}: {}", PID_FILE, e);
                                           }
                                        } else {
                                           println!("Process {} still running. You might need to use kill -9.", pid);
                                        }
                                    }
                                    Some(false) => {
                                        eprintln!("Error: Failed to send termination signal to process {} (signal not sent or process already terminating).", pid);
                                         s.refresh_processes();
                                        if s.process(pid).is_none() {
                                            println!("Server process {} seems to have already stopped.", pid);
                                            if let Err(e) = fs::remove_file(PID_FILE) {
                                               eprintln!("Warning: Could not remove PID file {}: {}", PID_FILE, e);
                                           }
                                        }
                                    }
                                    None => {
                                        eprintln!("Error: Failed to send termination signal to process {} (process may not exist or permissions issue for signalling).", pid);
                                        s.refresh_processes();
                                        if s.process(pid).is_none() {
                                            println!("Server process {} seems to have already stopped or does not exist.", pid);
                                            if let Err(e) = fs::remove_file(PID_FILE) {
                                               eprintln!("Warning: Could not remove PID file {}: {}", PID_FILE, e);
                                           }
                                        }
                                    }
                                }
                            } else {
                                println!("Process with PID {} not found. It might have already stopped.", pid);
                                if let Err(e) = fs::remove_file(PID_FILE) {
                                    eprintln!("Warning: Could not remove stale PID file {}: {}", PID_FILE, e);
                                }
                            }
                        }
                        Err(_) => eprintln!("Error: Invalid PID format in {}.", PID_FILE),
                    }
                }
                Err(_) => println!("Server not running (PID file {} not found).", PID_FILE),
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
                                println!("Server is running (PID: {}).", pid);
                                println!("Health check functionality is currently disabled.");
                            } else {
                                println!("Server is stopped (PID file {} found, but process {} not running).", PID_FILE, pid);
                                println!("Consider running 'rucho stop' to attempt cleanup or manually deleting {}.", PID_FILE);
                            }
                        }
                        Err(_) => eprintln!("Error: Invalid PID format in {}. Consider deleting it.", PID_FILE),
                    }
                }
                Err(_) => println!("Server is stopped (PID file {} not found).", PID_FILE),
            }
        }
        CliCommand::Version {} => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        }
    }
}

async fn run_server(config: &Config) {
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
        shutdown.await;
        tracing::info!("Shutdown signal received, all servers are stopping via shared handle.");
    } else {
        tracing::warn!("No server instances were configured or able to start.");
    }
}

async fn shutdown_signal(handle: Handle) {
    signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    tracing::info!("Signal received, starting graceful shutdown");
    handle.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
}
