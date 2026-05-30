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
    let builder = tracing_subscriber::fmt().with_max_level(log_level);
    match config.log_format.to_lowercase().as_str() {
        "json" => builder.json().init(),
        "text" => builder.init(),
        other => {
            eprintln!("Warning: Invalid log_format '{other}' in config, defaulting to text.");
            builder.init();
        }
    }

    // Dispatch command
    match args.command {
        CliCommand::Start {} => {
            // A PID-write failure is non-fatal (read-only FS, missing dir): the
            // server still starts and can be stopped with a signal.
            handle_start_command(&config.pid_file);

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
                config.request_id_enabled,
            );
            rucho::server::run_server(&config, app).await;
        }
        CliCommand::Stop {} => handle_stop_command(&config.pid_file),
        CliCommand::Status {} => handle_status_command(&config.pid_file),
        CliCommand::Version {} => handle_version_command(),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::fmt::MakeWriter;

    /// A `MakeWriter` that captures log output into a shared buffer.
    #[derive(Clone)]
    struct BufWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for BufWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().expect("buffer lock").extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for BufWriter {
        type Writer = BufWriter;
        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    /// Verifies that the `json` formatter (gated by the `json` cargo feature we
    /// enable for `log_format = json`) emits a parseable JSON log line. Uses a
    /// scoped subscriber so it doesn't fight the global default.
    #[test]
    fn json_log_format_emits_valid_json() {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let subscriber = tracing_subscriber::fmt()
            .json()
            .with_writer(BufWriter(buf.clone()))
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(unit = "test", "hello json");
        });

        let out = String::from_utf8(buf.lock().expect("buffer lock").clone()).unwrap();
        assert!(
            out.trim_start().starts_with('{'),
            "expected a JSON object, got: {out}"
        );
        let parsed: serde_json::Value =
            serde_json::from_str(out.trim()).expect("log line must be valid JSON");
        assert_eq!(parsed["level"], "INFO");
        assert_eq!(parsed["fields"]["message"], "hello json");
    }
}
