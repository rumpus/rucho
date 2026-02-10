//! Centralized constants for the Rucho application.
//!
//! This module contains all hardcoded values used throughout the application,
//! making it easier to maintain and modify configuration defaults.

/// Default installation prefix path.
pub const DEFAULT_PREFIX: &str = "/usr/local/rucho";

/// Default log level for the application.
pub const DEFAULT_LOG_LEVEL: &str = "info";

/// Default primary server listen address.
pub const DEFAULT_SERVER_LISTEN_PRIMARY: &str = "0.0.0.0:8080";

/// Default secondary server listen address.
pub const DEFAULT_SERVER_LISTEN_SECONDARY: &str = "0.0.0.0:9090";

/// Path to the PID file used for process management.
pub const PID_FILE_PATH: &str = "/var/run/rucho/rucho.pid";

/// Maximum delay allowed in seconds for the `/delay/:n` endpoint.
/// This prevents denial-of-service attacks by limiting how long a request can be held.
pub const MAX_DELAY_SECONDS: u64 = 300;

/// Maximum number of redirect hops allowed for the `/redirect/:n` endpoint.
/// This prevents abuse through excessively long redirect chains.
pub const MAX_REDIRECT_HOPS: u32 = 20;

/// Maximum buffer size in bytes for TCP/UDP connections.
/// This prevents memory exhaustion from malicious large payloads.
pub const MAX_BUFFER_SIZE: usize = 65536;

/// Default backoff duration in milliseconds for UDP error recovery.
pub const UDP_ERROR_BACKOFF_BASE_MS: u64 = 100;

/// Maximum backoff duration in milliseconds for UDP error recovery.
pub const UDP_ERROR_BACKOFF_MAX_MS: u64 = 5000;

/// Default HTTP keep-alive timeout in seconds.
/// Controls how long an idle HTTP connection stays open before closing.
pub const DEFAULT_HTTP_KEEP_ALIVE_TIMEOUT_SECS: u64 = 75;

/// Default TCP keep-alive idle time in seconds.
/// How long a connection must be idle before TCP probes start.
pub const DEFAULT_TCP_KEEPALIVE_SECS: u64 = 60;

/// Default TCP keep-alive probe interval in seconds.
/// Time between TCP keep-alive probe packets.
pub const DEFAULT_TCP_KEEPALIVE_INTERVAL_SECS: u64 = 15;

/// Default number of TCP keep-alive probe retries.
/// Number of failed probes before dropping the connection.
pub const DEFAULT_TCP_KEEPALIVE_RETRIES: u32 = 5;

/// Default header read timeout in seconds.
/// Maximum time to wait for a client to send complete request headers.
pub const DEFAULT_HEADER_READ_TIMEOUT_SECS: u64 = 30;
