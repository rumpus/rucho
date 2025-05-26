mod routes;
mod utils;

use crate::utils::config::Config; // Added import
use clap::Parser;
use std::fs;
use std::io::Write; // Read is unused
use std::process;
use std::str::FromStr; // Added import
use sysinfo::{Pid, Signal, System}; // SystemExt will be used via the System struct directly
use axum::Router; // Request and ServiceExt are unused
use tokio::{net::TcpListener, signal};
use tower_http::{
    cors::CorsLayer,
    normalize_path::NormalizePathLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level; // Added import
use tracing_subscriber;
use axum_server::Handle; // bind_rustls and Server are unused
// crate::utils::server_config::try_load_rustls_config will be used directly with crate:: prefix

// Temporarily comment out reqwest for build purposes
// use reqwest;

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

const PID_FILE: &str = "echo-server.pid";

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = Config::load_config(); // Load config

    // Initialize tracing subscriber with log level from config
    let log_level = Level::from_str(&config.log_level.to_uppercase())
        .unwrap_or_else(|_| {
            eprintln!("Warning: Invalid log level '{}' in config, defaulting to INFO.", config.log_level);
            Level::INFO
        });
    tracing_subscriber::fmt().with_max_level(log_level).init(); // Initialize tracing

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
            run_server(&config).await; // Pass config to run_server
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
                                println!("Stopping server (PID: {})...", pid);
                                match process.kill_with(Signal::Term) { // Handle Option<bool>
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
                            let mut s = System::new_all(); // SysInfoSystemExt is used here
                            s.refresh_processes();
                            if let Some(_process) = s.process(pid) { 
                                println!("Server is running (PID: {}).", pid);
                                println!("Health check functionality is currently disabled.");
                            } else {
                                println!("Server is stopped (PID file {} found, but process {} not running).", PID_FILE, pid);
                                println!("Consider running 'echo-server stop' to attempt cleanup or manually deleting {}.", PID_FILE);
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

async fn run_server(config: &Config) { // Takes config as an argument
    // tracing_subscriber::fmt::init(); // This is now done in main

    let handle = Handle::new();
    let shutdown = shutdown_signal(handle.clone());

    let app = Router::new()
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

    if let Some(rustls_config) = crate::utils::server_config::try_load_rustls_config().await { // Corrected path
        tracing::info!("Starting HTTPS server on https://0.0.0.0:8443");
        axum_server::bind_rustls("0.0.0.0:8443".parse().unwrap(), rustls_config)
            .handle(handle)
            .serve(app.clone().into_make_service())
            .await
            .unwrap();
    } else {
        let listener1 = TcpListener::bind(&config.server_listen_primary).await.unwrap_or_else(|e| {
            eprintln!("Error binding to primary address {}: {}", config.server_listen_primary, e);
            process::exit(1);
        });
        let listener2 = TcpListener::bind(&config.server_listen_secondary).await.unwrap_or_else(|e| {
            eprintln!("Error binding to secondary address {}: {}", config.server_listen_secondary, e);
            process::exit(1);
        });
        let std_listener1 = listener1.into_std().unwrap();
        let std_listener2 = listener2.into_std().unwrap();
        tracing::info!("Starting HTTP servers on {} and {}", config.server_listen_primary, config.server_listen_secondary);
        let serve1 = axum_server::Server::from_tcp(std_listener1).serve(app.clone().into_make_service());
        let serve2 = axum_server::Server::from_tcp(std_listener2).serve(app.into_make_service());

        tokio::select! {
            _ = serve1 => {},
            _ = serve2 => {},
            _ = shutdown => {
                tracing::info!("Shutting down server");
            }
        }
    }
}

async fn shutdown_signal(handle: Handle) {
    signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    tracing::info!("Signal received, starting graceful shutdown");
    handle.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
}
