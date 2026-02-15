use std::env;
use std::fs;
use std::path::PathBuf;

use crate::utils::constants::{
    DEFAULT_HEADER_READ_TIMEOUT_SECS, DEFAULT_HTTP_KEEP_ALIVE_TIMEOUT_SECS, DEFAULT_LOG_LEVEL,
    DEFAULT_PREFIX, DEFAULT_SERVER_LISTEN_PRIMARY, DEFAULT_SERVER_LISTEN_SECONDARY,
    DEFAULT_TCP_KEEPALIVE_INTERVAL_SECS, DEFAULT_TCP_KEEPALIVE_RETRIES, DEFAULT_TCP_KEEPALIVE_SECS,
};

/// Configuration for chaos engineering mode.
///
/// Chaos mode enables random injection of failures, delays, and response corruption
/// to help test application resilience. Each chaos type is configured independently
/// and rolls against its own probability rate per request.
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    /// Active chaos types (e.g., "failure", "delay", "corruption").
    pub modes: Vec<String>,
    /// Probability of injecting a failure response (0.01-1.0).
    pub failure_rate: f64,
    /// HTTP status codes to randomly return on failure (e.g., [500, 502, 503]).
    pub failure_codes: Vec<u16>,
    /// Probability of injecting a delay (0.01-1.0).
    pub delay_rate: f64,
    /// Delay duration in milliseconds, or "random" for random delays.
    pub delay_ms: String,
    /// Maximum delay in milliseconds when delay_ms is "random".
    pub delay_max_ms: u64,
    /// Probability of corrupting the response body (0.01-1.0).
    pub corruption_rate: f64,
    /// How to corrupt the response body: "empty", "truncate", or "garbage".
    pub corruption_type: String,
    /// Whether to add X-Chaos header to affected responses (default: true).
    pub inform_header: bool,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        ChaosConfig {
            modes: Vec::new(),
            failure_rate: 0.0,
            failure_codes: Vec::new(),
            delay_rate: 0.0,
            delay_ms: String::new(),
            delay_max_ms: 0,
            corruption_rate: 0.0,
            corruption_type: String::new(),
            inform_header: true,
        }
    }
}

impl ChaosConfig {
    /// Returns true if any chaos mode is enabled.
    pub fn is_enabled(&self) -> bool {
        !self.modes.is_empty()
    }

    /// Returns true if failure injection is enabled.
    pub fn has_failure(&self) -> bool {
        self.modes.iter().any(|m| m == "failure")
    }

    /// Returns true if delay injection is enabled.
    pub fn has_delay(&self) -> bool {
        self.modes.iter().any(|m| m == "delay")
    }

    /// Returns true if response corruption is enabled.
    pub fn has_corruption(&self) -> bool {
        self.modes.iter().any(|m| m == "corruption")
    }
}

/// Macro to load an environment variable into a config field.
///
/// Accepts an `$env_reader` callable (e.g. `env::var` or a test mock) so that
/// tests can supply a pure HashMap-backed reader instead of mutating the process
/// environment.
macro_rules! load_env_var {
    ($config:expr, $field:ident, $env_var:expr, $env_reader:expr) => {
        if let Ok(value) = $env_reader($env_var) {
            $config.$field = value;
        }
    };
    ($config:expr, $field:ident, $env_var:expr, $env_reader:expr, option) => {
        if let Ok(value) = $env_reader($env_var) {
            $config.$field = Some(value);
        }
    };
    ($config:expr, $field:ident, $env_var:expr, $env_reader:expr, bool) => {
        if let Ok(value) = $env_reader($env_var) {
            $config.$field = value.eq_ignore_ascii_case("true") || value == "1";
        }
    };
    ($config:expr, $field:ident, $env_var:expr, $env_reader:expr, u64) => {
        if let Ok(value) = $env_reader($env_var) {
            if let Ok(v) = value.parse::<u64>() {
                $config.$field = v;
            }
        }
    };
    ($config:expr, $field:ident, $env_var:expr, $env_reader:expr, u32) => {
        if let Ok(value) = $env_reader($env_var) {
            if let Ok(v) = value.parse::<u32>() {
                $config.$field = v;
            }
        }
    };
}

/// Holds the application configuration.
///
/// Configuration values are loaded in the following order of precedence (lowest to highest):
/// 1. Hardcoded default values.
/// 2. Values from the system-wide configuration file at `/etc/rucho/rucho.conf` (if it exists).
/// 3. Values from the local configuration file at `./rucho.conf` in the current working directory (if it exists).
/// 4. Environment variables prefixed with `RUCHO_` (e.g., `RUCHO_PREFIX`).
///
/// A sample configuration file, `rucho.conf.default`, can be found in the `config_samples`
/// directory of the source repository. This can be used as a template for creating
/// `/etc/rucho/rucho.conf` or `./rucho.conf`.
#[derive(Debug, Clone)]
pub struct Config {
    /// Prefix for certain operations, e.g., file paths (Not actively used by server logic yet).
    pub prefix: String,
    /// Logging level for the application (e.g., "info", "debug", "warn", "error").
    pub log_level: String,
    /// Primary listen address and port for the server (e.g., "0.0.0.0:8080" or "ssl:0.0.0.0:8443").
    pub server_listen_primary: String,
    /// Secondary listen address and port for the server (e.g., "0.0.0.0:9090" or "ssl:0.0.0.0:9443"). Can be empty.
    pub server_listen_secondary: String,
    /// Optional TCP echo listener address (e.g., "0.0.0.0:7777").
    pub server_listen_tcp: Option<String>,
    /// Optional UDP echo listener address (e.g., "0.0.0.0:7778").
    pub server_listen_udp: Option<String>,
    /// Optional path to an SSL certificate file for HTTPS. Required if any listen address uses "ssl:".
    pub ssl_cert: Option<String>,
    /// Optional path to an SSL private key file for HTTPS. Required if any listen address uses "ssl:".
    pub ssl_key: Option<String>,
    /// Enable the /metrics endpoint for request statistics.
    pub metrics_enabled: bool,
    /// Enable response compression (gzip, brotli) based on client Accept-Encoding.
    pub compression_enabled: bool,
    /// HTTP keep-alive timeout in seconds. How long an idle connection stays open.
    pub http_keep_alive_timeout: u64,
    /// TCP keep-alive idle time in seconds. How long before probes start on idle connections.
    pub tcp_keepalive_time: u64,
    /// TCP keep-alive probe interval in seconds.
    pub tcp_keepalive_interval: u64,
    /// Number of TCP keep-alive probe retries before dropping the connection.
    pub tcp_keepalive_retries: u32,
    /// Disable Nagle's algorithm for lower latency on small responses.
    pub tcp_nodelay: bool,
    /// Maximum time in seconds to wait for request headers from a client.
    pub header_read_timeout: u64,
    /// Chaos engineering configuration.
    pub chaos: ChaosConfig,
}

impl Default for Config {
    /// Provides the hardcoded default configuration values for the application.
    /// These defaults are the first layer in the configuration loading process.
    fn default() -> Self {
        Config {
            prefix: DEFAULT_PREFIX.to_string(),
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            server_listen_primary: DEFAULT_SERVER_LISTEN_PRIMARY.to_string(),
            server_listen_secondary: DEFAULT_SERVER_LISTEN_SECONDARY.to_string(),
            server_listen_tcp: None,
            server_listen_udp: None,
            ssl_cert: None,
            ssl_key: None,
            metrics_enabled: false,
            compression_enabled: false,
            http_keep_alive_timeout: DEFAULT_HTTP_KEEP_ALIVE_TIMEOUT_SECS,
            tcp_keepalive_time: DEFAULT_TCP_KEEPALIVE_SECS,
            tcp_keepalive_interval: DEFAULT_TCP_KEEPALIVE_INTERVAL_SECS,
            tcp_keepalive_retries: DEFAULT_TCP_KEEPALIVE_RETRIES,
            tcp_nodelay: true,
            header_read_timeout: DEFAULT_HEADER_READ_TIMEOUT_SECS,
            chaos: ChaosConfig::default(),
        }
    }
}

/// Errors that can occur during configuration validation.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValidationError {
    /// SSL certificate specified without key
    SslCertWithoutKey,
    /// SSL key specified without certificate
    SslKeyWithoutCert,
    /// A connection tuning value is invalid
    Connection(String),
    /// A chaos configuration requirement is not met
    Chaos(String),
}

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigValidationError::SslCertWithoutKey => {
                write!(f, "SSL certificate specified without key")
            }
            ConfigValidationError::SslKeyWithoutCert => {
                write!(f, "SSL key specified without certificate")
            }
            ConfigValidationError::Connection(msg) => {
                write!(f, "Connection config error: {}", msg)
            }
            ConfigValidationError::Chaos(msg) => {
                write!(f, "Chaos config error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConfigValidationError {}

impl Config {
    // Internal helper function to parse lines from a configuration file.
    // It updates the provided `Config` mutable instance with values found in the `contents`.
    // Lines starting with '#' or empty lines are ignored.
    // Expected format for lines is "key = value".
    #[cfg_attr(not(test), allow(dead_code))] // Allow dead code for this helper when not in test builds
    fn parse_file_contents(config: &mut Config, contents: String) {
        for line in contents.lines() {
            // Skip comments and empty lines
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                let value = parts[1].trim();
                match key {
                    "prefix" => config.prefix = value.to_string(),
                    "log_level" => config.log_level = value.to_string(),
                    "server_listen_primary" => config.server_listen_primary = value.to_string(),
                    "server_listen_secondary" => config.server_listen_secondary = value.to_string(),
                    "server_listen_tcp" => config.server_listen_tcp = Some(value.to_string()),
                    "server_listen_udp" => config.server_listen_udp = Some(value.to_string()),
                    "ssl_cert" => config.ssl_cert = Some(value.to_string()),
                    "ssl_key" => config.ssl_key = Some(value.to_string()),
                    "metrics_enabled" => {
                        config.metrics_enabled = value.eq_ignore_ascii_case("true") || value == "1"
                    }
                    "compression_enabled" => {
                        config.compression_enabled =
                            value.eq_ignore_ascii_case("true") || value == "1"
                    }
                    "http_keep_alive_timeout" => {
                        if let Ok(v) = value.parse::<u64>() {
                            config.http_keep_alive_timeout = v;
                        }
                    }
                    "tcp_keepalive_time" => {
                        if let Ok(v) = value.parse::<u64>() {
                            config.tcp_keepalive_time = v;
                        }
                    }
                    "tcp_keepalive_interval" => {
                        if let Ok(v) = value.parse::<u64>() {
                            config.tcp_keepalive_interval = v;
                        }
                    }
                    "tcp_keepalive_retries" => {
                        if let Ok(v) = value.parse::<u32>() {
                            config.tcp_keepalive_retries = v;
                        }
                    }
                    "tcp_nodelay" => {
                        config.tcp_nodelay = value.eq_ignore_ascii_case("true") || value == "1"
                    }
                    "header_read_timeout" => {
                        if let Ok(v) = value.parse::<u64>() {
                            config.header_read_timeout = v;
                        }
                    }
                    "chaos_mode" => {
                        config.chaos.modes = value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                    "chaos_failure_rate" => {
                        if let Ok(v) = value.parse::<f64>() {
                            config.chaos.failure_rate = v;
                        }
                    }
                    "chaos_failure_codes" => {
                        config.chaos.failure_codes = value
                            .split(',')
                            .filter_map(|s| s.trim().parse::<u16>().ok())
                            .collect();
                    }
                    "chaos_delay_rate" => {
                        if let Ok(v) = value.parse::<f64>() {
                            config.chaos.delay_rate = v;
                        }
                    }
                    "chaos_delay_ms" => {
                        config.chaos.delay_ms = value.to_string();
                    }
                    "chaos_delay_max_ms" => {
                        if let Ok(v) = value.parse::<u64>() {
                            config.chaos.delay_max_ms = v;
                        }
                    }
                    "chaos_corruption_rate" => {
                        if let Ok(v) = value.parse::<f64>() {
                            config.chaos.corruption_rate = v;
                        }
                    }
                    "chaos_corruption_type" => {
                        config.chaos.corruption_type = value.to_string();
                    }
                    "chaos_inform_header" => {
                        config.chaos.inform_header =
                            value.eq_ignore_ascii_case("true") || value == "1"
                    }
                    _ => eprintln!("Warning: Unknown key in config file: {}", key),
                }
            } else {
                eprintln!("Warning: Invalid line in config file: {}", line);
            }
        }
    }

    /// Loads configuration from file paths with an injectable environment reader.
    ///
    /// This is the core loading method. Tests inject a mock `env_reader` to avoid
    /// mutating process-global environment variables; production code passes `env::var`.
    ///
    /// Loading order (later stages override earlier ones):
    /// 1. Defaults from `Config::default()`.
    /// 2. Values from the ETC path (or `/etc/rucho/rucho.conf`).
    /// 3. Values from the local path (or `./rucho.conf`).
    /// 4. Environment variables via `env_reader`.
    #[cfg_attr(not(test), allow(dead_code))]
    fn load_from_paths_with_env(
        etc_path_override: Option<PathBuf>,
        local_path_override: Option<PathBuf>,
        env_reader: &dyn Fn(&str) -> Result<String, env::VarError>,
    ) -> Self {
        let mut config = Config::default();

        // Determine paths to use: override or default.
        let etc_config_path =
            etc_path_override.unwrap_or_else(|| PathBuf::from("/etc/rucho/rucho.conf"));
        let local_config_path = local_path_override.unwrap_or_else(|| PathBuf::from("rucho.conf"));

        // Load from the system-wide config file (e.g., /etc/rucho/rucho.conf or override)
        if etc_config_path.exists() {
            if let Ok(contents) = fs::read_to_string(&etc_config_path) {
                Self::parse_file_contents(&mut config, contents);
            } else {
                eprintln!(
                    "Warning: Could not read system config file at {:?}, though it exists.",
                    etc_config_path
                );
            }
        }

        // Load from the local config file (e.g., ./rucho.conf or override), overriding previous values
        if local_config_path.exists() {
            if let Ok(contents) = fs::read_to_string(&local_config_path) {
                Self::parse_file_contents(&mut config, contents);
            } else {
                eprintln!(
                    "Warning: Could not read local config file at {:?}, though it exists.",
                    local_config_path
                );
            }
        }

        // 4. Override with environment variables
        load_env_var!(config, prefix, "RUCHO_PREFIX", env_reader);
        load_env_var!(config, log_level, "RUCHO_LOG_LEVEL", env_reader);
        load_env_var!(
            config,
            server_listen_primary,
            "RUCHO_SERVER_LISTEN_PRIMARY",
            env_reader
        );
        load_env_var!(
            config,
            server_listen_secondary,
            "RUCHO_SERVER_LISTEN_SECONDARY",
            env_reader
        );
        load_env_var!(
            config,
            server_listen_tcp,
            "RUCHO_SERVER_LISTEN_TCP",
            env_reader,
            option
        );
        load_env_var!(
            config,
            server_listen_udp,
            "RUCHO_SERVER_LISTEN_UDP",
            env_reader,
            option
        );
        load_env_var!(config, ssl_cert, "RUCHO_SSL_CERT", env_reader, option);
        load_env_var!(config, ssl_key, "RUCHO_SSL_KEY", env_reader, option);
        load_env_var!(
            config,
            metrics_enabled,
            "RUCHO_METRICS_ENABLED",
            env_reader,
            bool
        );
        load_env_var!(
            config,
            compression_enabled,
            "RUCHO_COMPRESSION_ENABLED",
            env_reader,
            bool
        );
        load_env_var!(
            config,
            http_keep_alive_timeout,
            "RUCHO_HTTP_KEEP_ALIVE_TIMEOUT",
            env_reader,
            u64
        );
        load_env_var!(
            config,
            tcp_keepalive_time,
            "RUCHO_TCP_KEEPALIVE_TIME",
            env_reader,
            u64
        );
        load_env_var!(
            config,
            tcp_keepalive_interval,
            "RUCHO_TCP_KEEPALIVE_INTERVAL",
            env_reader,
            u64
        );
        load_env_var!(
            config,
            tcp_keepalive_retries,
            "RUCHO_TCP_KEEPALIVE_RETRIES",
            env_reader,
            u32
        );
        load_env_var!(config, tcp_nodelay, "RUCHO_TCP_NODELAY", env_reader, bool);
        load_env_var!(
            config,
            header_read_timeout,
            "RUCHO_HEADER_READ_TIMEOUT",
            env_reader,
            u64
        );

        // Chaos mode env vars (manual parsing since macro doesn't support nested fields)
        if let Ok(value) = env_reader("RUCHO_CHAOS_MODE") {
            config.chaos.modes = value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        if let Ok(value) = env_reader("RUCHO_CHAOS_FAILURE_RATE") {
            if let Ok(v) = value.parse::<f64>() {
                config.chaos.failure_rate = v;
            }
        }
        if let Ok(value) = env_reader("RUCHO_CHAOS_FAILURE_CODES") {
            config.chaos.failure_codes = value
                .split(',')
                .filter_map(|s| s.trim().parse::<u16>().ok())
                .collect();
        }
        if let Ok(value) = env_reader("RUCHO_CHAOS_DELAY_RATE") {
            if let Ok(v) = value.parse::<f64>() {
                config.chaos.delay_rate = v;
            }
        }
        if let Ok(value) = env_reader("RUCHO_CHAOS_DELAY_MS") {
            config.chaos.delay_ms = value;
        }
        if let Ok(value) = env_reader("RUCHO_CHAOS_DELAY_MAX_MS") {
            if let Ok(v) = value.parse::<u64>() {
                config.chaos.delay_max_ms = v;
            }
        }
        if let Ok(value) = env_reader("RUCHO_CHAOS_CORRUPTION_RATE") {
            if let Ok(v) = value.parse::<f64>() {
                config.chaos.corruption_rate = v;
            }
        }
        if let Ok(value) = env_reader("RUCHO_CHAOS_CORRUPTION_TYPE") {
            config.chaos.corruption_type = value;
        }
        if let Ok(value) = env_reader("RUCHO_CHAOS_INFORM_HEADER") {
            config.chaos.inform_header = value.eq_ignore_ascii_case("true") || value == "1";
        }

        config
    }

    /// Loads configuration from file paths using real environment variables.
    #[cfg_attr(not(test), allow(dead_code))]
    fn load_from_paths(
        etc_path_override: Option<PathBuf>,
        local_path_override: Option<PathBuf>,
    ) -> Self {
        Self::load_from_paths_with_env(etc_path_override, local_path_override, &|key| env::var(key))
    }

    /// Validates the configuration for consistency.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the configuration is valid, or a `ConfigValidationError` if not.
    ///
    /// # Errors
    ///
    /// - `SslCertWithoutKey`: SSL certificate is specified but key is missing
    /// - `SslKeyWithoutCert`: SSL key is specified but certificate is missing
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        match (&self.ssl_cert, &self.ssl_key) {
            (Some(_), None) => return Err(ConfigValidationError::SslCertWithoutKey),
            (None, Some(_)) => return Err(ConfigValidationError::SslKeyWithoutCert),
            _ => {}
        }

        self.validate_connection()?;
        self.validate_chaos()?;

        Ok(())
    }

    /// Validates connection keep-alive and timeout settings.
    fn validate_connection(&self) -> Result<(), ConfigValidationError> {
        if self.http_keep_alive_timeout == 0 {
            return Err(ConfigValidationError::Connection(
                "http_keep_alive_timeout must be greater than 0".to_string(),
            ));
        }
        if self.tcp_keepalive_time == 0 {
            return Err(ConfigValidationError::Connection(
                "tcp_keepalive_time must be greater than 0".to_string(),
            ));
        }
        if self.tcp_keepalive_interval == 0 {
            return Err(ConfigValidationError::Connection(
                "tcp_keepalive_interval must be greater than 0".to_string(),
            ));
        }
        if self.tcp_keepalive_retries == 0 || self.tcp_keepalive_retries > 10 {
            return Err(ConfigValidationError::Connection(
                "tcp_keepalive_retries must be between 1 and 10".to_string(),
            ));
        }
        if self.header_read_timeout == 0 {
            return Err(ConfigValidationError::Connection(
                "header_read_timeout must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }

    /// Validates the chaos engineering configuration.
    ///
    /// Checks that all required sub-configs are present for each enabled chaos type,
    /// rates are within valid ranges, and values are well-formed.
    fn validate_chaos(&self) -> Result<(), ConfigValidationError> {
        let chaos = &self.chaos;

        if !chaos.is_enabled() {
            return Ok(());
        }

        // Check for unknown chaos types
        let valid_types = ["failure", "delay", "corruption"];
        for mode in &chaos.modes {
            if !valid_types.contains(&mode.as_str()) {
                return Err(ConfigValidationError::Chaos(format!(
                    "Unknown chaos type '{}'. Valid types: failure, delay, corruption",
                    mode
                )));
            }
        }

        // Validate failure config
        if chaos.has_failure() {
            if chaos.failure_rate < 0.01 || chaos.failure_rate > 1.0 {
                return Err(ConfigValidationError::Chaos(
                    "chaos_failure_rate must be between 0.01 and 1.0".to_string(),
                ));
            }
            if chaos.failure_codes.is_empty() {
                return Err(ConfigValidationError::Chaos(
                    "chaos_failure_codes is required when failure mode is enabled".to_string(),
                ));
            }
            for &code in &chaos.failure_codes {
                if !(400..=599).contains(&code) {
                    return Err(ConfigValidationError::Chaos(format!(
                        "Invalid failure code {}. Must be between 400 and 599",
                        code
                    )));
                }
            }
        }

        // Validate delay config
        if chaos.has_delay() {
            if chaos.delay_rate < 0.01 || chaos.delay_rate > 1.0 {
                return Err(ConfigValidationError::Chaos(
                    "chaos_delay_rate must be between 0.01 and 1.0".to_string(),
                ));
            }
            if chaos.delay_ms.is_empty() {
                return Err(ConfigValidationError::Chaos(
                    "chaos_delay_ms is required when delay mode is enabled".to_string(),
                ));
            }
            if chaos.delay_ms == "random" {
                if chaos.delay_max_ms == 0 {
                    return Err(ConfigValidationError::Chaos(
                        "chaos_delay_max_ms is required when chaos_delay_ms is 'random'"
                            .to_string(),
                    ));
                }
            } else if chaos.delay_ms.parse::<u64>().is_err() {
                return Err(ConfigValidationError::Chaos(
                    "chaos_delay_ms must be a number or 'random'".to_string(),
                ));
            }
        }

        // Validate corruption config
        if chaos.has_corruption() {
            if chaos.corruption_rate < 0.01 || chaos.corruption_rate > 1.0 {
                return Err(ConfigValidationError::Chaos(
                    "chaos_corruption_rate must be between 0.01 and 1.0".to_string(),
                ));
            }
            let valid_corruption_types = ["empty", "truncate", "garbage"];
            if !valid_corruption_types.contains(&chaos.corruption_type.as_str()) {
                return Err(ConfigValidationError::Chaos(format!(
                    "Invalid chaos_corruption_type '{}'. Valid types: empty, truncate, garbage",
                    chaos.corruption_type
                )));
            }
        }

        Ok(())
    }

    /// Loads the configuration for the application.
    ///
    /// It applies configurations in the following order (later stages override earlier ones):
    /// 1. Sets hardcoded default values.
    /// 2. Attempts to read and apply settings from `/etc/rucho/rucho.conf`.
    /// 3. Attempts to read and apply settings from `./rucho.conf` (current working directory).
    /// 4. Applies any settings from environment variables prefixed with `RUCHO_`.
    ///
    /// The configuration files (`/etc/rucho/rucho.conf`, `./rucho.conf`) should contain
    /// `key = value` pairs, one per line. Lines starting with `#` are comments.
    ///
    /// Refer to `config_samples/rucho.conf.default` for a template.
    ///
    /// Supported keys in config files and corresponding environment variables:
    /// - `prefix` (`RUCHO_PREFIX`)
    /// - `log_level` (`RUCHO_LOG_LEVEL`)
    /// - `server_listen_primary` (`RUCHO_SERVER_LISTEN_PRIMARY`)
    /// - `server_listen_secondary` (`RUCHO_SERVER_LISTEN_SECONDARY`)
    /// - `server_listen_tcp` (`RUCHO_SERVER_LISTEN_TCP`)
    /// - `server_listen_udp` (`RUCHO_SERVER_LISTEN_UDP`)
    /// - `ssl_cert` (`RUCHO_SSL_CERT`)
    /// - `ssl_key` (`RUCHO_SSL_KEY`)
    /// - `metrics_enabled` (`RUCHO_METRICS_ENABLED`)
    /// - `compression_enabled` (`RUCHO_COMPRESSION_ENABLED`)
    pub fn load() -> Self {
        Self::load_from_paths(None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Manages temporary directories for config file tests.
    ///
    /// Unlike the previous version, this does NOT mutate process-global state
    /// (no `env::set_var`, no `env::set_current_dir`). Tests pass explicit file
    /// paths and a mock env reader instead.
    struct TestEnv {
        _etc_dir: TempDir,
        etc_rucho_conf_path: PathBuf,
        _cwd_dir: TempDir,
        cwd_rucho_conf_path: PathBuf,
    }

    impl TestEnv {
        fn new() -> Self {
            let etc_dir = TempDir::new().expect("Failed to create temp etc dir");
            let etc_rucho_dir = etc_dir.path().join("rucho");
            fs::create_dir_all(&etc_rucho_dir).expect("Failed to create fake /etc/rucho");

            let cwd_dir = TempDir::new().expect("Failed to create temp cwd dir");

            TestEnv {
                etc_rucho_conf_path: etc_rucho_dir.join("rucho.conf"),
                _etc_dir: etc_dir,
                cwd_rucho_conf_path: cwd_dir.path().join("rucho.conf"),
                _cwd_dir: cwd_dir,
            }
        }

        fn create_config_file(&self, path: &std::path::Path, content: &str) {
            let mut file = File::create(path)
                .unwrap_or_else(|e| panic!("Failed to create config file at {:?}: {}", path, e));
            writeln!(file, "{}", content).unwrap();
        }

        fn non_existent_etc(&self) -> PathBuf {
            self.etc_rucho_conf_path
                .parent()
                .unwrap()
                .join("non_existent.conf")
        }

        fn non_existent_cwd(&self) -> PathBuf {
            self.cwd_rucho_conf_path
                .parent()
                .unwrap()
                .join("non_existent.conf")
        }
    }

    /// Returns an env reader backed by the given HashMap.
    fn mock_env(vars: HashMap<&str, &str>) -> impl Fn(&str) -> Result<String, env::VarError> {
        let owned: HashMap<String, String> = vars
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |key: &str| owned.get(key).cloned().ok_or(env::VarError::NotPresent)
    }

    /// Returns an env reader that always returns `NotPresent`.
    fn empty_env() -> impl Fn(&str) -> Result<String, env::VarError> {
        |_: &str| Err(env::VarError::NotPresent)
    }

    #[test]
    fn test_default_config() {
        let env = empty_env();
        let non_existent_etc = PathBuf::from("/tmp/non_existent_default_test_etc.conf");
        let non_existent_cwd = PathBuf::from("/tmp/non_existent_default_test_cwd.conf");
        let config =
            Config::load_from_paths_with_env(Some(non_existent_etc), Some(non_existent_cwd), &env);

        assert_eq!(config.prefix, "/usr/local/rucho");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.server_listen_primary, "0.0.0.0:8080");
        assert_eq!(config.server_listen_secondary, "0.0.0.0:9090");
        assert_eq!(config.ssl_cert, None);
        assert_eq!(config.ssl_key, None);
    }

    #[test]
    fn test_load_from_etc_only() {
        let t = TestEnv::new();
        t.create_config_file(
            &t.etc_rucho_conf_path,
            "prefix = /etc/path\nlog_level = etc_level",
        );

        let env = empty_env();
        let config = Config::load_from_paths_with_env(
            Some(t.etc_rucho_conf_path.clone()),
            Some(t.non_existent_cwd()),
            &env,
        );

        assert_eq!(config.prefix, "/etc/path");
        assert_eq!(config.log_level, "etc_level");
        assert_eq!(config.server_listen_primary, "0.0.0.0:8080");
    }

    #[test]
    fn test_load_from_cwd_only() {
        let t = TestEnv::new();
        t.create_config_file(
            &t.cwd_rucho_conf_path,
            "prefix = /cwd/path\nlog_level = cwd_level",
        );

        let env = empty_env();
        let config = Config::load_from_paths_with_env(
            Some(t.non_existent_etc()),
            Some(t.cwd_rucho_conf_path.clone()),
            &env,
        );

        assert_eq!(config.prefix, "/cwd/path");
        assert_eq!(config.log_level, "cwd_level");
        assert_eq!(config.server_listen_secondary, "0.0.0.0:9090");
    }

    #[test]
    fn test_cwd_overrides_etc() {
        let t = TestEnv::new();
        t.create_config_file(
            &t.etc_rucho_conf_path,
            "prefix = /etc/path\nlog_level = etc_level",
        );
        t.create_config_file(
            &t.cwd_rucho_conf_path,
            "prefix = /cwd/path\nserver_listen_primary = 1.1.1.1:1111",
        );

        let env = empty_env();
        let config = Config::load_from_paths_with_env(
            Some(t.etc_rucho_conf_path.clone()),
            Some(t.cwd_rucho_conf_path.clone()),
            &env,
        );

        assert_eq!(config.prefix, "/cwd/path");
        assert_eq!(config.log_level, "etc_level");
        assert_eq!(config.server_listen_primary, "1.1.1.1:1111");
        assert_eq!(config.server_listen_secondary, "0.0.0.0:9090");
    }

    #[test]
    fn test_env_overrides_all_files() {
        let t = TestEnv::new();
        t.create_config_file(
            &t.etc_rucho_conf_path,
            "prefix = /etc/path\nlog_level = etc_level",
        );
        t.create_config_file(
            &t.cwd_rucho_conf_path,
            "prefix = /cwd/path\nlog_level = cwd_level",
        );

        let env = mock_env(HashMap::from([
            ("RUCHO_PREFIX", "/env/path"),
            ("RUCHO_LOG_LEVEL", "env_level"),
            ("RUCHO_SERVER_LISTEN_PRIMARY", "env_primary"),
        ]));
        let config = Config::load_from_paths_with_env(
            Some(t.etc_rucho_conf_path.clone()),
            Some(t.cwd_rucho_conf_path.clone()),
            &env,
        );

        assert_eq!(config.prefix, "/env/path");
        assert_eq!(config.log_level, "env_level");
        assert_eq!(config.server_listen_primary, "env_primary");
        assert_eq!(config.server_listen_secondary, "0.0.0.0:9090");
    }

    #[test]
    fn test_partial_configs_layering() {
        let t = TestEnv::new();
        t.create_config_file(
            &t.etc_rucho_conf_path,
            "prefix = /etc/path\nlog_level = etc_level_original",
        );
        t.create_config_file(
            &t.cwd_rucho_conf_path,
            "log_level = cwd_level\nserver_listen_secondary = 2.2.2.2:2222",
        );

        let env = mock_env(HashMap::from([(
            "RUCHO_SERVER_LISTEN_PRIMARY",
            "env_primary",
        )]));
        let config = Config::load_from_paths_with_env(
            Some(t.etc_rucho_conf_path.clone()),
            Some(t.cwd_rucho_conf_path.clone()),
            &env,
        );

        assert_eq!(config.prefix, "/etc/path");
        assert_eq!(config.log_level, "cwd_level");
        assert_eq!(config.server_listen_primary, "env_primary");
        assert_eq!(config.server_listen_secondary, "2.2.2.2:2222");
    }

    #[test]
    fn test_load_ssl_from_file() {
        let t = TestEnv::new();
        t.create_config_file(
            &t.cwd_rucho_conf_path,
            "ssl_cert = /test/cert.pem\nssl_key = /test/key.pem",
        );

        let env = empty_env();
        let config = Config::load_from_paths_with_env(
            Some(t.non_existent_etc()),
            Some(t.cwd_rucho_conf_path.clone()),
            &env,
        );

        assert_eq!(config.ssl_cert, Some("/test/cert.pem".to_string()));
        assert_eq!(config.ssl_key, Some("/test/key.pem".to_string()));
    }

    #[test]
    fn test_load_ssl_from_env() {
        let env = mock_env(HashMap::from([
            ("RUCHO_SSL_CERT", "/env/cert.pem"),
            ("RUCHO_SSL_KEY", "/env/key.pem"),
        ]));
        let non_existent_etc = PathBuf::from("/tmp/non_existent_ssl_env_etc.conf");
        let non_existent_cwd = PathBuf::from("/tmp/non_existent_ssl_env_cwd.conf");
        let config =
            Config::load_from_paths_with_env(Some(non_existent_etc), Some(non_existent_cwd), &env);

        assert_eq!(config.ssl_cert, Some("/env/cert.pem".to_string()));
        assert_eq!(config.ssl_key, Some("/env/key.pem".to_string()));
    }

    #[test]
    fn test_env_overrides_file_for_ssl() {
        let t = TestEnv::new();
        t.create_config_file(
            &t.cwd_rucho_conf_path,
            "ssl_cert = /file/cert.pem\nssl_key = /file/key.pem",
        );

        let env = mock_env(HashMap::from([
            ("RUCHO_SSL_CERT", "/env/cert.pem"),
            ("RUCHO_SSL_KEY", "/env/key.pem"),
        ]));
        let config = Config::load_from_paths_with_env(
            Some(t.non_existent_etc()),
            Some(t.cwd_rucho_conf_path.clone()),
            &env,
        );

        assert_eq!(config.ssl_cert, Some("/env/cert.pem".to_string()));
        assert_eq!(config.ssl_key, Some("/env/key.pem".to_string()));
    }

    #[test]
    fn test_partial_ssl_config_layering() {
        let t = TestEnv::new();
        t.create_config_file(&t.cwd_rucho_conf_path, "ssl_cert = /file/cert.pem");

        let env = mock_env(HashMap::from([("RUCHO_SSL_KEY", "/env/key.pem")]));
        let config = Config::load_from_paths_with_env(
            Some(t.non_existent_etc()),
            Some(t.cwd_rucho_conf_path.clone()),
            &env,
        );

        assert_eq!(config.ssl_cert, Some("/file/cert.pem".to_string()));
        assert_eq!(config.ssl_key, Some("/env/key.pem".to_string()));
    }

    #[test]
    fn test_load_tcp_udp_from_file() {
        let t = TestEnv::new();
        t.create_config_file(
            &t.cwd_rucho_conf_path,
            "server_listen_tcp = 127.0.0.1:1234\nserver_listen_udp = 127.0.0.1:5678",
        );

        let env = empty_env();
        let config = Config::load_from_paths_with_env(
            Some(t.non_existent_etc()),
            Some(t.cwd_rucho_conf_path.clone()),
            &env,
        );

        assert_eq!(config.server_listen_tcp, Some("127.0.0.1:1234".to_string()));
        assert_eq!(config.server_listen_udp, Some("127.0.0.1:5678".to_string()));
    }

    #[test]
    fn test_load_tcp_udp_from_env() {
        let env = mock_env(HashMap::from([
            ("RUCHO_SERVER_LISTEN_TCP", "127.0.0.1:1234"),
            ("RUCHO_SERVER_LISTEN_UDP", "127.0.0.1:5678"),
        ]));
        let non_existent_etc = PathBuf::from("/tmp/non_existent_tcp_udp_env_etc.conf");
        let non_existent_cwd = PathBuf::from("/tmp/non_existent_tcp_udp_env_cwd.conf");
        let config =
            Config::load_from_paths_with_env(Some(non_existent_etc), Some(non_existent_cwd), &env);

        assert_eq!(config.server_listen_tcp, Some("127.0.0.1:1234".to_string()));
        assert_eq!(config.server_listen_udp, Some("127.0.0.1:5678".to_string()));
    }

    #[test]
    fn test_env_overrides_file_for_tcp_udp() {
        let t = TestEnv::new();
        t.create_config_file(
            &t.cwd_rucho_conf_path,
            "server_listen_tcp = /file/tcp\nserver_listen_udp = /file/udp",
        );

        let env = mock_env(HashMap::from([
            ("RUCHO_SERVER_LISTEN_TCP", "/env/tcp"),
            ("RUCHO_SERVER_LISTEN_UDP", "/env/udp"),
        ]));
        let config = Config::load_from_paths_with_env(
            Some(t.non_existent_etc()),
            Some(t.cwd_rucho_conf_path.clone()),
            &env,
        );

        assert_eq!(config.server_listen_tcp, Some("/env/tcp".to_string()));
        assert_eq!(config.server_listen_udp, Some("/env/udp".to_string()));
    }

    #[test]
    fn test_partial_tcp_udp_config_layering() {
        let t = TestEnv::new();
        t.create_config_file(&t.cwd_rucho_conf_path, "server_listen_tcp = /file/tcp");

        let env = mock_env(HashMap::from([("RUCHO_SERVER_LISTEN_UDP", "/env/udp")]));
        let config = Config::load_from_paths_with_env(
            Some(t.non_existent_etc()),
            Some(t.cwd_rucho_conf_path.clone()),
            &env,
        );

        assert_eq!(config.server_listen_tcp, Some("/file/tcp".to_string()));
        assert_eq!(config.server_listen_udp, Some("/env/udp".to_string()));
    }

    #[test]
    fn test_validate_both_ssl_options_present() {
        let config = Config {
            ssl_cert: Some("/path/to/cert.pem".to_string()),
            ssl_key: Some("/path/to/key.pem".to_string()),
            ..Config::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_both_ssl_options_absent() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_ssl_cert_without_key() {
        let config = Config {
            ssl_cert: Some("/path/to/cert.pem".to_string()),
            ssl_key: None,
            ..Config::default()
        };
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::SslCertWithoutKey)
        );
    }

    #[test]
    fn test_validate_ssl_key_without_cert() {
        let config = Config {
            ssl_cert: None,
            ssl_key: Some("/path/to/key.pem".to_string()),
            ..Config::default()
        };
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::SslKeyWithoutCert)
        );
    }

    #[test]
    fn test_compression_enabled_default_false() {
        let env = empty_env();
        let non_existent_etc = PathBuf::from("/tmp/non_existent_compression_test_etc.conf");
        let non_existent_cwd = PathBuf::from("/tmp/non_existent_compression_test_cwd.conf");
        let config =
            Config::load_from_paths_with_env(Some(non_existent_etc), Some(non_existent_cwd), &env);

        assert!(!config.compression_enabled);
    }

    #[test]
    fn test_load_compression_enabled_from_file() {
        let t = TestEnv::new();
        t.create_config_file(&t.cwd_rucho_conf_path, "compression_enabled = true");

        let env = empty_env();
        let config = Config::load_from_paths_with_env(
            Some(t.non_existent_etc()),
            Some(t.cwd_rucho_conf_path.clone()),
            &env,
        );

        assert!(config.compression_enabled);
    }

    #[test]
    fn test_load_compression_enabled_from_env() {
        let env = mock_env(HashMap::from([("RUCHO_COMPRESSION_ENABLED", "true")]));
        let non_existent_etc = PathBuf::from("/tmp/non_existent_compression_env_etc.conf");
        let non_existent_cwd = PathBuf::from("/tmp/non_existent_compression_env_cwd.conf");
        let config =
            Config::load_from_paths_with_env(Some(non_existent_etc), Some(non_existent_cwd), &env);

        assert!(config.compression_enabled);
    }

    #[test]
    fn test_env_overrides_file_for_compression() {
        let t = TestEnv::new();
        t.create_config_file(&t.cwd_rucho_conf_path, "compression_enabled = false");

        let env = mock_env(HashMap::from([("RUCHO_COMPRESSION_ENABLED", "true")]));
        let config = Config::load_from_paths_with_env(
            Some(t.non_existent_etc()),
            Some(t.cwd_rucho_conf_path.clone()),
            &env,
        );

        assert!(config.compression_enabled);
    }

    // --- Chaos config tests ---

    #[test]
    fn test_chaos_disabled_by_default() {
        let env = empty_env();
        let non_existent_etc = PathBuf::from("/tmp/non_existent_chaos_default_etc.conf");
        let non_existent_cwd = PathBuf::from("/tmp/non_existent_chaos_default_cwd.conf");
        let config =
            Config::load_from_paths_with_env(Some(non_existent_etc), Some(non_existent_cwd), &env);

        assert!(!config.chaos.is_enabled());
        assert!(config.chaos.modes.is_empty());
        assert!(config.chaos.inform_header); // default true
    }

    #[test]
    fn test_chaos_config_from_file() {
        let t = TestEnv::new();
        t.create_config_file(
            &t.cwd_rucho_conf_path,
            "chaos_mode = failure,delay\n\
             chaos_failure_rate = 0.5\n\
             chaos_failure_codes = 500,503\n\
             chaos_delay_rate = 0.3\n\
             chaos_delay_ms = 2000\n\
             chaos_inform_header = false",
        );

        let env = empty_env();
        let config = Config::load_from_paths_with_env(
            Some(t.non_existent_etc()),
            Some(t.cwd_rucho_conf_path.clone()),
            &env,
        );

        assert!(config.chaos.is_enabled());
        assert_eq!(config.chaos.modes, vec!["failure", "delay"]);
        assert!((config.chaos.failure_rate - 0.5).abs() < f64::EPSILON);
        assert_eq!(config.chaos.failure_codes, vec![500, 503]);
        assert!((config.chaos.delay_rate - 0.3).abs() < f64::EPSILON);
        assert_eq!(config.chaos.delay_ms, "2000");
        assert!(!config.chaos.inform_header);
    }

    #[test]
    fn test_chaos_config_from_env() {
        let env = mock_env(HashMap::from([
            ("RUCHO_CHAOS_MODE", "corruption"),
            ("RUCHO_CHAOS_CORRUPTION_RATE", "0.1"),
            ("RUCHO_CHAOS_CORRUPTION_TYPE", "empty"),
        ]));
        let non_existent_etc = PathBuf::from("/tmp/non_existent_chaos_env_etc.conf");
        let non_existent_cwd = PathBuf::from("/tmp/non_existent_chaos_env_cwd.conf");
        let config =
            Config::load_from_paths_with_env(Some(non_existent_etc), Some(non_existent_cwd), &env);

        assert!(config.chaos.is_enabled());
        assert!(config.chaos.has_corruption());
        assert!((config.chaos.corruption_rate - 0.1).abs() < f64::EPSILON);
        assert_eq!(config.chaos.corruption_type, "empty");
    }

    #[test]
    fn test_chaos_env_overrides_file() {
        let t = TestEnv::new();
        t.create_config_file(
            &t.cwd_rucho_conf_path,
            "chaos_mode = failure\n\
             chaos_failure_rate = 0.5\n\
             chaos_failure_codes = 500",
        );

        let env = mock_env(HashMap::from([("RUCHO_CHAOS_FAILURE_RATE", "0.9")]));
        let config = Config::load_from_paths_with_env(
            Some(t.non_existent_etc()),
            Some(t.cwd_rucho_conf_path.clone()),
            &env,
        );

        assert!((config.chaos.failure_rate - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_chaos_validate_valid_failure_config() {
        let mut config = Config::default();
        config.chaos.modes = vec!["failure".to_string()];
        config.chaos.failure_rate = 0.5;
        config.chaos.failure_codes = vec![500, 503];

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_chaos_validate_missing_failure_codes() {
        let mut config = Config::default();
        config.chaos.modes = vec!["failure".to_string()];
        config.chaos.failure_rate = 0.5;
        // failure_codes empty

        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::Chaos(_))
        ));
    }

    #[test]
    fn test_chaos_validate_failure_rate_too_low() {
        let mut config = Config::default();
        config.chaos.modes = vec!["failure".to_string()];
        config.chaos.failure_rate = 0.001;
        config.chaos.failure_codes = vec![500];

        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::Chaos(_))
        ));
    }

    #[test]
    fn test_chaos_validate_failure_rate_too_high() {
        let mut config = Config::default();
        config.chaos.modes = vec!["failure".to_string()];
        config.chaos.failure_rate = 1.5;
        config.chaos.failure_codes = vec![500];

        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::Chaos(_))
        ));
    }

    #[test]
    fn test_chaos_validate_invalid_failure_code() {
        let mut config = Config::default();
        config.chaos.modes = vec!["failure".to_string()];
        config.chaos.failure_rate = 0.5;
        config.chaos.failure_codes = vec![200]; // not 400-599

        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::Chaos(_))
        ));
    }

    #[test]
    fn test_chaos_validate_missing_delay_ms() {
        let mut config = Config::default();
        config.chaos.modes = vec!["delay".to_string()];
        config.chaos.delay_rate = 0.5;
        // delay_ms empty

        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::Chaos(_))
        ));
    }

    #[test]
    fn test_chaos_validate_random_delay_missing_max() {
        let mut config = Config::default();
        config.chaos.modes = vec!["delay".to_string()];
        config.chaos.delay_rate = 0.5;
        config.chaos.delay_ms = "random".to_string();
        config.chaos.delay_max_ms = 0;

        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::Chaos(_))
        ));
    }

    #[test]
    fn test_chaos_validate_valid_delay_random() {
        let mut config = Config::default();
        config.chaos.modes = vec!["delay".to_string()];
        config.chaos.delay_rate = 0.5;
        config.chaos.delay_ms = "random".to_string();
        config.chaos.delay_max_ms = 3000;

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_chaos_validate_invalid_corruption_type() {
        let mut config = Config::default();
        config.chaos.modes = vec!["corruption".to_string()];
        config.chaos.corruption_rate = 0.5;
        config.chaos.corruption_type = "invalid".to_string();

        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::Chaos(_))
        ));
    }

    #[test]
    fn test_chaos_validate_valid_corruption() {
        let mut config = Config::default();
        config.chaos.modes = vec!["corruption".to_string()];
        config.chaos.corruption_rate = 0.5;
        config.chaos.corruption_type = "truncate".to_string();

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_chaos_validate_unknown_type() {
        let mut config = Config::default();
        config.chaos.modes = vec!["unknown".to_string()];

        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::Chaos(_))
        ));
    }

    #[test]
    fn test_chaos_validate_disabled_skips_validation() {
        let config = Config::default();
        // chaos is disabled by default, should pass even without sub-configs
        assert!(config.validate().is_ok());
    }

    // --- Connection keep-alive config tests ---

    #[test]
    fn test_connection_defaults() {
        let env = empty_env();
        let non_existent_etc = PathBuf::from("/tmp/non_existent_conn_default_etc.conf");
        let non_existent_cwd = PathBuf::from("/tmp/non_existent_conn_default_cwd.conf");
        let config =
            Config::load_from_paths_with_env(Some(non_existent_etc), Some(non_existent_cwd), &env);

        assert_eq!(config.http_keep_alive_timeout, 75);
        assert_eq!(config.tcp_keepalive_time, 60);
        assert_eq!(config.tcp_keepalive_interval, 15);
        assert_eq!(config.tcp_keepalive_retries, 5);
        assert!(config.tcp_nodelay);
        assert_eq!(config.header_read_timeout, 30);
    }

    #[test]
    fn test_connection_config_from_file() {
        let t = TestEnv::new();
        t.create_config_file(
            &t.cwd_rucho_conf_path,
            "http_keep_alive_timeout = 120\n\
             tcp_keepalive_time = 90\n\
             tcp_keepalive_interval = 20\n\
             tcp_keepalive_retries = 3\n\
             tcp_nodelay = false\n\
             header_read_timeout = 45",
        );

        let env = empty_env();
        let config = Config::load_from_paths_with_env(
            Some(t.non_existent_etc()),
            Some(t.cwd_rucho_conf_path.clone()),
            &env,
        );

        assert_eq!(config.http_keep_alive_timeout, 120);
        assert_eq!(config.tcp_keepalive_time, 90);
        assert_eq!(config.tcp_keepalive_interval, 20);
        assert_eq!(config.tcp_keepalive_retries, 3);
        assert!(!config.tcp_nodelay);
        assert_eq!(config.header_read_timeout, 45);
    }

    #[test]
    fn test_connection_config_from_env() {
        let env = mock_env(HashMap::from([
            ("RUCHO_HTTP_KEEP_ALIVE_TIMEOUT", "100"),
            ("RUCHO_TCP_KEEPALIVE_TIME", "80"),
            ("RUCHO_TCP_KEEPALIVE_INTERVAL", "10"),
            ("RUCHO_TCP_KEEPALIVE_RETRIES", "8"),
            ("RUCHO_TCP_NODELAY", "false"),
            ("RUCHO_HEADER_READ_TIMEOUT", "60"),
        ]));
        let non_existent_etc = PathBuf::from("/tmp/non_existent_conn_env_etc.conf");
        let non_existent_cwd = PathBuf::from("/tmp/non_existent_conn_env_cwd.conf");
        let config =
            Config::load_from_paths_with_env(Some(non_existent_etc), Some(non_existent_cwd), &env);

        assert_eq!(config.http_keep_alive_timeout, 100);
        assert_eq!(config.tcp_keepalive_time, 80);
        assert_eq!(config.tcp_keepalive_interval, 10);
        assert_eq!(config.tcp_keepalive_retries, 8);
        assert!(!config.tcp_nodelay);
        assert_eq!(config.header_read_timeout, 60);
    }

    #[test]
    fn test_connection_env_overrides_file() {
        let t = TestEnv::new();
        t.create_config_file(
            &t.cwd_rucho_conf_path,
            "http_keep_alive_timeout = 50\n\
             tcp_keepalive_time = 40",
        );

        let env = mock_env(HashMap::from([("RUCHO_HTTP_KEEP_ALIVE_TIMEOUT", "200")]));
        let config = Config::load_from_paths_with_env(
            Some(t.non_existent_etc()),
            Some(t.cwd_rucho_conf_path.clone()),
            &env,
        );

        assert_eq!(config.http_keep_alive_timeout, 200); // env wins
        assert_eq!(config.tcp_keepalive_time, 40); // file value
    }

    #[test]
    fn test_validate_connection_defaults_pass() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_http_keep_alive_timeout_zero() {
        let config = Config {
            http_keep_alive_timeout: 0,
            ..Config::default()
        };
        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::Connection(_))
        ));
    }

    #[test]
    fn test_validate_tcp_keepalive_time_zero() {
        let config = Config {
            tcp_keepalive_time: 0,
            ..Config::default()
        };
        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::Connection(_))
        ));
    }

    #[test]
    fn test_validate_tcp_keepalive_interval_zero() {
        let config = Config {
            tcp_keepalive_interval: 0,
            ..Config::default()
        };
        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::Connection(_))
        ));
    }

    #[test]
    fn test_validate_tcp_keepalive_retries_zero() {
        let config = Config {
            tcp_keepalive_retries: 0,
            ..Config::default()
        };
        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::Connection(_))
        ));
    }

    #[test]
    fn test_validate_tcp_keepalive_retries_too_high() {
        let config = Config {
            tcp_keepalive_retries: 11,
            ..Config::default()
        };
        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::Connection(_))
        ));
    }

    #[test]
    fn test_validate_tcp_keepalive_retries_boundary() {
        let config = Config {
            tcp_keepalive_retries: 1,
            ..Config::default()
        };
        assert!(config.validate().is_ok());

        let config = Config {
            tcp_keepalive_retries: 10,
            ..Config::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_header_read_timeout_zero() {
        let config = Config {
            header_read_timeout: 0,
            ..Config::default()
        };
        assert!(matches!(
            config.validate(),
            Err(ConfigValidationError::Connection(_))
        ));
    }
}
