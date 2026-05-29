//! Rucho server entry point.
//!
//! This is the main entry point for the Rucho application. It handles CLI argument
//! parsing and dispatches to the appropriate command handlers. The Axum app itself
//! is assembled by [`rucho::app::build_app`].

use std::str::FromStr;
use std::sync::Arc;

use clap::Parser;
use tracing::Level;

use rucho::app::build_app;
use rucho::cli::{
    commands::{
        handle_start_command, handle_status_command, handle_stop_command, handle_version_command,
    },
    Args, CliCommand,
};
use rucho::utils::config::Config;
use rucho::utils::metrics::Metrics;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = Config::load();

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    }

    // Initialize tracing with configured log level
    let log_level = Level::from_str(&config.log_level.to_uppercase()).unwrap_or_else(|_| {
        eprintln!(
            "Warning: Invalid log level '{}' in config, defaulting to INFO.",
            config.log_level
        );
        Level::INFO
    });
    tracing_subscriber::fmt().with_max_level(log_level).init();

    // Dispatch command
    match args.command {
        CliCommand::Start {} => {
            if handle_start_command() {
                // Create metrics store if enabled
                let metrics = if config.metrics_enabled {
                    tracing::info!("Metrics endpoint enabled at /metrics");
                    Some(Arc::new(Metrics::new()))
                } else {
                    None
                };

                tracing::info!(
                    "Connection settings: TCP keep-alive={}s, TCP nodelay={}, HTTP timeout={}s, header timeout={}s",
                    config.tcp_keepalive_time,
                    config.tcp_nodelay,
                    config.http_keep_alive_timeout,
                    config.header_read_timeout,
                );

                // Log chaos mode if enabled
                if config.chaos.is_enabled() {
                    tracing::info!("Chaos mode enabled: {}", config.chaos.modes.join(", "));
                }

                let chaos = Arc::new(config.chaos.clone());
                let app = build_app(
                    metrics,
                    config.compression_enabled,
                    chaos,
                    config.max_body_size_bytes,
                );
                rucho::server::run_server(&config, app).await;
            }
        }
        CliCommand::Stop {} => handle_stop_command(),
        CliCommand::Status {} => handle_status_command(),
        CliCommand::Version {} => handle_version_command(),
    }
}
