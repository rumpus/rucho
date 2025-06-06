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
/// Represents the OpenAPI documentation structure.
///
/// This struct is used by `utoipa` to generate the OpenAPI specification
/// for the Rucho API. It aggregates all the paths and components
/// (schemas, responses, etc.) that are part of the API.
struct ApiDoc;

/// Path to the file storing the PID of the running Rucho server.
/// Used for managing the server process (e.g., stopping, checking status).
const PID_FILE: &str = "/var/run/rucho/rucho.pid";

/// The main entry point for the Rucho application.
///
/// Parses command line arguments, initializes configuration and logging,
/// and executes the appropriate command.
#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = Config::load(); // Load config

    // Initialize tracing subscriber with log level from config
    let log_level = Level::from_str(&config.log_level.to_uppercase())
        .unwrap_or_else(|_| {
            eprintln!("Warning: Invalid log level '{}' in config, defaulting to INFO.", config.log_level);
            Level::INFO
        });
    tracing_subscriber::fmt().with_max_level(log_level).init(); // Initialize tracing

    // Dispatch command
    match args.command {
        CliCommand::Start {} => {
            // Handle Start command
            println!("Starting server...");
            let pid = process::id();
            // Create PID file
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
            run_server(&config).await; // Pass config to run_server
        }
        CliCommand::Stop {} => {
            // Handle Stop command
            match fs::read_to_string(PID_FILE) {
                Ok(pid_str) => {
                    // PID file exists, try to parse PID
                    match pid_str.trim().parse::<usize>() {
                        Ok(pid_val) => {
                            let pid = Pid::from(pid_val);
                            let mut s = System::new_all(); // SysInfoSystemExt is used here
                            s.refresh_processes(); // Refresh process list
                            // Check if process exists
                            if let Some(process) = s.process(pid) {
                                println!("Stopping server (PID: {})...", pid);
                                // Attempt to kill the process
                                match process.kill_with(Signal::Term) { // Handle Option<bool>
                                    Some(true) => {
                                        println!("Termination signal sent to process {}.", pid);
                                        // Wait a bit for the process to terminate
                                        std::thread::sleep(std::time::Duration::from_secs(1));
                                        s.refresh_processes(); // Refresh again
                                        if s.process(pid).is_none() {
                                           println!("Server stopped successfully.");
                                           // Attempt to remove PID file
                                           if let Err(e) = fs::remove_file(PID_FILE) {
                                               eprintln!("Warning: Could not remove PID file {}: {}", PID_FILE, e);
                                           }
                                        } else {
                                           println!("Process {} still running. You might need to use kill -9.", pid);
                                        }
                                    }
                                    Some(false) => {
                                        eprintln!("Error: Failed to send termination signal to process {} (signal not sent or process already terminating).", pid);
                                         s.refresh_processes(); // Refresh to check current status
                                        if s.process(pid).is_none() {
                                            println!("Server process {} seems to have already stopped.", pid);
                                            if let Err(e) = fs::remove_file(PID_FILE) {
                                               eprintln!("Warning: Could not remove PID file {}: {}", PID_FILE, e);
                                           }
                                        }
                                    }
                                    None => {
                                        // This case means the signal could not be sent, possibly due to permissions or the process not existing.
                                        eprintln!("Error: Failed to send termination signal to process {} (process may not exist or permissions issue for signalling).", pid);
                                        s.refresh_processes(); // Refresh to check current status
                                        if s.process(pid).is_none() {
                                            println!("Server process {} seems to have already stopped or does not exist.", pid);
                                            // Clean up PID file if process is gone
                                            if let Err(e) = fs::remove_file(PID_FILE) {
                                               eprintln!("Warning: Could not remove PID file {}: {}", PID_FILE, e);
                                           }
                                        }
                                    }
                                }
                            } else {
                                // Process not found, but PID file exists
                                println!("Process with PID {} not found. It might have already stopped.", pid);
                                // Attempt to remove stale PID file
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
            // Handle Status command
            match fs::read_to_string(PID_FILE) {
                Ok(pid_str) => {
                    // PID file exists, try to parse PID
                    match pid_str.trim().parse::<usize>() {
                        Ok(pid_val) => {
                            let pid = Pid::from(pid_val);
                            let mut s = System::new_all(); // SysInfoSystemExt is used here
                            s.refresh_processes(); // Refresh process list
                            // Check if process exists
                            if let Some(_process) = s.process(pid) {
                                println!("Server is running (PID: {}).", pid);
                                // TODO: Implement actual health check endpoint call
                                println!("Health check functionality is currently disabled.");
                            } else {
                                // Process not found, but PID file exists
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
            // Handle Version command
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        }
    }
}

/// Runs the Axum web server with the provided configuration.
///
/// This function sets up the HTTP/S listeners, configures routing,
/// and handles graceful shutdown.
async fn run_server(config: &Config) { // Takes config as an argument
    // tracing_subscriber::fmt::init(); // This is now done in main

    // Create a new Axum server handle for graceful shutdown.
    let handle = Handle::new();
    // Spawn a task to listen for shutdown signals (e.g., Ctrl+C).
    // Pass a clone of the handle to the shutdown signal listener.
    let shutdown = shutdown_signal(handle.clone());

    // Define the main application router by merging various route modules.
    // Also, sets up Swagger UI.
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi())) // Swagger UI endpoint
        .merge(routes::core_routes::router()) // Core application routes
        .merge(routes::healthz::router()) // Health check route
        .merge(routes::delay::router())   // Delay testing route
        // Apply middleware layers.
        // TraceLayer for logging requests and responses.
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        // CorsLayer for handling Cross-Origin Resource Sharing.
        .layer(CorsLayer::permissive())
        // NormalizePathLayer to trim trailing slashes from request paths.
        .layer(NormalizePathLayer::trim_trailing_slash());

    // Prepare a list of listener addresses (IP:port) and SSL status from config.
    let mut listeners_to_start: Vec<(String, bool)> = Vec::new();

    // Parse primary listen address.
    if let Some(parsed) = crate::utils::server_config::parse_listen_address(&config.server_listen_primary) {
        listeners_to_start.push(parsed);
    }
    // Parse secondary listen address, if configured.
    if let Some(parsed) = crate::utils::server_config::parse_listen_address(&config.server_listen_secondary) {
        listeners_to_start.push(parsed);
    }

    // Store handles for each spawned server task.
    let mut server_handles: Vec<tokio::task::JoinHandle<Result<(), std::io::Error>>> = Vec::new();

    // Iterate over parsed listener configurations and start servers.
    for (address_str, is_ssl) in listeners_to_start {
        let app_clone = app.clone(); // Clone the app router for each server instance.
        let handle_clone = handle.clone(); // Clone the server handle for each server instance.

        // Attempt to parse the string address into a SocketAddr.
        let sock_addr: std::net::SocketAddr = match address_str.parse() {
            Ok(addr) => addr,
            Err(e) => {
                tracing::error!("Failed to parse address '{}': {}. Skipping this listener.", address_str, e);
                continue; // Skip to the next listener if parsing fails.
            }
        };

        if is_ssl {
            // Configure and start an HTTPS server.
            // Attempt to load SSL certificate and key.
            match crate::utils::server_config::try_load_rustls_config(config.ssl_cert.as_deref(), config.ssl_key.as_deref()).await {
                Some(rustls_config) => {
                    tracing::info!("Starting HTTPS server on https://{}", sock_addr);
                    // Bind the server with Rustls configuration.
                    let server_future = axum_server::bind_rustls(sock_addr, rustls_config)
                        .handle(handle_clone) // Attach the graceful shutdown handle.
                        .serve(app_clone.into_make_service()); // Serve the Axum app.
                    server_handles.push(tokio::spawn(server_future)); // Spawn the server task.
                }
                None => {
                    // Log an error if SSL config loading fails.
                    tracing::error!("Failed to load Rustls config for {}: HTTPS server not started. Check SSL certificate/key configuration and paths.", sock_addr);
                }
            }
        } else {
            // Configure and start an HTTP server.
            // Attempt to bind a TCP listener to the address.
            match tokio::net::TcpListener::bind(sock_addr).await {
                Ok(listener) => {
                    // Convert Tokio TcpListener to std::net::TcpListener for axum_server.
                    match listener.into_std() {
                        Ok(std_listener) => {
                            tracing::info!("Starting HTTP server on http://{}", sock_addr);
                            // Create the server from the standard TCP listener.
                            let server_future = axum_server::Server::from_tcp(std_listener)
                                .handle(handle_clone) // Attach the graceful shutdown handle.
                                .serve(app_clone.into_make_service()); // Serve the Axum app.
                            server_handles.push(tokio::spawn(server_future)); // Spawn the server task.
                        }
                        Err(e) => {
                             tracing::error!("Failed to convert tokio listener to std for {}: {}. Skipping this listener.", sock_addr, e);
                        }
                    }
                }
                Err(e) => {
                    // Log an error if binding the HTTP listener fails.
                    tracing::error!("Failed to bind HTTP listener for {}: {}. Skipping this listener.", sock_addr, e);
                }
            }
        }
    }

    // Check if any server instances were successfully started.
    if !server_handles.is_empty() {
        tracing::info!("{} server(s) started. Waiting for shutdown signal...", server_handles.len());
        // Wait for the shutdown signal (e.g., Ctrl+C).
        shutdown.await; // This is `shutdown_signal(handle.clone())` passed from main
        tracing::info!("Shutdown signal received, all servers are stopping via shared handle.");
    } else {
        // Log a warning if no servers could be started (e.g., due to config errors).
        tracing::warn!("No server instances were configured or able to start.");
    }
}

/// Listens for a Ctrl+C signal to initiate a graceful shutdown of the server.
async fn shutdown_signal(handle: Handle) {
    signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    tracing::info!("Signal received, starting graceful shutdown");
    handle.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
}
