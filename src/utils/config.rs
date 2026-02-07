use std::env;
use std::fs;
use std::path::PathBuf;

use crate::utils::constants::{
    DEFAULT_LOG_LEVEL, DEFAULT_PREFIX, DEFAULT_SERVER_LISTEN_PRIMARY,
    DEFAULT_SERVER_LISTEN_SECONDARY,
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
macro_rules! load_env_var {
    ($config:expr, $field:ident, $env_var:expr) => {
        if let Ok(value) = env::var($env_var) {
            $config.$field = value;
        }
    };
    ($config:expr, $field:ident, $env_var:expr, option) => {
        if let Ok(value) = env::var($env_var) {
            $config.$field = Some(value);
        }
    };
    ($config:expr, $field:ident, $env_var:expr, bool) => {
        if let Ok(value) = env::var($env_var) {
            $config.$field = value.eq_ignore_ascii_case("true") || value == "1";
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

    /// Loads configuration by attempting to read from specified file paths and then
    /// applying environment variable overrides. This function is primarily intended for
    /// testing purposes, allowing explicit control over which configuration files are loaded.
    ///
    /// If `etc_path_override` is `None`, it defaults to `/etc/rucho/rucho.conf`.
    /// If `local_path_override` is `None`, it defaults to `./rucho.conf` (relative to current CWD).
    ///
    /// The loading order within this function is:
    /// 1. Defaults from `Config::default()`.
    /// 2. Values from the ETC path override (or default ETC path).
    /// 3. Values from the local path override (or default local path), potentially overriding ETC values.
    /// 4. Environment variables, overriding any values set by files.
    #[cfg_attr(not(test), allow(dead_code))] // Allow dead code for this helper when not in test builds
    fn load_from_paths(
        etc_path_override: Option<PathBuf>,
        local_path_override: Option<PathBuf>,
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
                // Log a warning if the file exists but cannot be read
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
                // Log a warning if the file exists but cannot be read
                eprintln!(
                    "Warning: Could not read local config file at {:?}, though it exists.",
                    local_config_path
                );
            }
        }

        // 4. Override with environment variables
        load_env_var!(config, prefix, "RUCHO_PREFIX");
        load_env_var!(config, log_level, "RUCHO_LOG_LEVEL");
        load_env_var!(config, server_listen_primary, "RUCHO_SERVER_LISTEN_PRIMARY");
        load_env_var!(
            config,
            server_listen_secondary,
            "RUCHO_SERVER_LISTEN_SECONDARY"
        );
        load_env_var!(config, server_listen_tcp, "RUCHO_SERVER_LISTEN_TCP", option);
        load_env_var!(config, server_listen_udp, "RUCHO_SERVER_LISTEN_UDP", option);
        load_env_var!(config, ssl_cert, "RUCHO_SSL_CERT", option);
        load_env_var!(config, ssl_key, "RUCHO_SSL_KEY", option);
        load_env_var!(config, metrics_enabled, "RUCHO_METRICS_ENABLED", bool);
        load_env_var!(
            config,
            compression_enabled,
            "RUCHO_COMPRESSION_ENABLED",
            bool
        );

        // Chaos mode env vars (manual parsing since macro doesn't support nested fields)
        if let Ok(value) = env::var("RUCHO_CHAOS_MODE") {
            config.chaos.modes = value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        if let Ok(value) = env::var("RUCHO_CHAOS_FAILURE_RATE") {
            if let Ok(v) = value.parse::<f64>() {
                config.chaos.failure_rate = v;
            }
        }
        if let Ok(value) = env::var("RUCHO_CHAOS_FAILURE_CODES") {
            config.chaos.failure_codes = value
                .split(',')
                .filter_map(|s| s.trim().parse::<u16>().ok())
                .collect();
        }
        if let Ok(value) = env::var("RUCHO_CHAOS_DELAY_RATE") {
            if let Ok(v) = value.parse::<f64>() {
                config.chaos.delay_rate = v;
            }
        }
        if let Ok(value) = env::var("RUCHO_CHAOS_DELAY_MS") {
            config.chaos.delay_ms = value;
        }
        if let Ok(value) = env::var("RUCHO_CHAOS_DELAY_MAX_MS") {
            if let Ok(v) = value.parse::<u64>() {
                config.chaos.delay_max_ms = v;
            }
        }
        if let Ok(value) = env::var("RUCHO_CHAOS_CORRUPTION_RATE") {
            if let Ok(v) = value.parse::<f64>() {
                config.chaos.corruption_rate = v;
            }
        }
        if let Ok(value) = env::var("RUCHO_CHAOS_CORRUPTION_TYPE") {
            config.chaos.corruption_type = value;
        }
        if let Ok(value) = env::var("RUCHO_CHAOS_INFORM_HEADER") {
            config.chaos.inform_header = value.eq_ignore_ascii_case("true") || value == "1";
        }

        config
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

        self.validate_chaos()?;

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
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Module for configuration loading tests.
    // TestEnv struct helps manage temporary directories and files for testing config loading.
    /// Sets up a controlled environment for testing configuration loading.
    ///
    /// This involves creating temporary directories to simulate `/etc/rucho/` and a
    /// temporary current working directory (CWD). It also handles cleaning up
    /// environment variables and restoring the original CWD when dropped.
    struct TestEnv {
        _etc_dir: TempDir,            // Temporary directory for simulating /etc
        etc_rucho_conf_path: PathBuf, // Path to the simulated /etc/rucho/rucho.conf
        _cwd_dir: TempDir,            // Temporary directory for simulating the CWD
        cwd_rucho_conf_path: PathBuf, // Path to the simulated ./rucho.conf in the temp CWD
        original_cwd: PathBuf,        // Stores the original CWD to restore it later
    }

    impl TestEnv {
        /// Creates a new `TestEnv` instance.
        ///
        /// This sets up temporary directories for `/etc/rucho` and a test-specific CWD.
        /// It also changes the current directory to the temporary CWD.
        fn new() -> Self {
            // Clear all RUCHO env vars at the START to ensure test isolation
            // (tests may run in parallel, so we can't rely only on Drop cleanup)
            Self::clear_env_vars();

            let etc_dir = TempDir::new().expect("Failed to create temp etc dir");
            let etc_rucho_dir = etc_dir.path().join("rucho"); // Simulate /etc/rucho
            fs::create_dir_all(&etc_rucho_dir).expect("Failed to create fake /etc/rucho");

            let cwd_dir = TempDir::new().expect("Failed to create temp cwd dir");

            let original_cwd = env::current_dir().unwrap();
            env::set_current_dir(cwd_dir.path()).unwrap();

            TestEnv {
                etc_rucho_conf_path: etc_rucho_dir.join("rucho.conf"),
                _etc_dir: etc_dir, // Keep TempDir to ensure it's cleaned up on drop
                cwd_rucho_conf_path: cwd_dir.path().join("rucho.conf"), // Path to ./rucho.conf in temp CWD
                _cwd_dir: cwd_dir,                                      // Keep TempDir for CWD
                original_cwd,
            }
        }

        fn clear_env_vars() {
            env::remove_var("RUCHO_PREFIX");
            env::remove_var("RUCHO_LOG_LEVEL");
            env::remove_var("RUCHO_SERVER_LISTEN_PRIMARY");
            env::remove_var("RUCHO_SERVER_LISTEN_SECONDARY");
            env::remove_var("RUCHO_SERVER_LISTEN_TCP");
            env::remove_var("RUCHO_SERVER_LISTEN_UDP");
            env::remove_var("RUCHO_SSL_CERT");
            env::remove_var("RUCHO_SSL_KEY");
            env::remove_var("RUCHO_METRICS_ENABLED");
            env::remove_var("RUCHO_COMPRESSION_ENABLED");
            env::remove_var("RUCHO_CHAOS_MODE");
            env::remove_var("RUCHO_CHAOS_FAILURE_RATE");
            env::remove_var("RUCHO_CHAOS_FAILURE_CODES");
            env::remove_var("RUCHO_CHAOS_DELAY_RATE");
            env::remove_var("RUCHO_CHAOS_DELAY_MS");
            env::remove_var("RUCHO_CHAOS_DELAY_MAX_MS");
            env::remove_var("RUCHO_CHAOS_CORRUPTION_RATE");
            env::remove_var("RUCHO_CHAOS_CORRUPTION_TYPE");
            env::remove_var("RUCHO_CHAOS_INFORM_HEADER");
        }

        fn create_config_file(&self, path: &std::path::Path, content: &str) {
            let mut file = File::create(path)
                .unwrap_or_else(|e| panic!("Failed to create config file at {:?}: {}", path, e));
            writeln!(file, "{}", content).unwrap();
        }
    }

    /// Restores the original CWD and cleans up environment variables set during tests.
    impl Drop for TestEnv {
        fn drop(&mut self) {
            // Attempt to restore CWD but don't panic if it fails (e.g., if directory was deleted)
            let _ = env::set_current_dir(&self.original_cwd);
            // TempDirs will be automatically removed when they go out of scope.
            Self::clear_env_vars();
        }
    }

    #[test]
    fn test_default_config() {
        let _env = TestEnv::new(); // Sets up CWD, cleans up vars.
                                   // To test defaults, we call load_from_paths with paths that are guaranteed not to exist.
                                   // This ensures that only the hardcoded defaults are loaded.
        let non_existent_etc =
            PathBuf::from("/tmp/non_existent_rucho_config_for_default_test_etc.conf");
        let non_existent_cwd =
            PathBuf::from("./non_existent_rucho_config_for_default_test_cwd.conf");
        let config = Config::load_from_paths(Some(non_existent_etc), Some(non_existent_cwd));

        // Assert that all configuration values match the hardcoded defaults.
        assert_eq!(config.prefix, "/usr/local/rucho");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.server_listen_primary, "0.0.0.0:8080");
        assert_eq!(config.server_listen_secondary, "0.0.0.0:9090");
        assert_eq!(config.ssl_cert, None);
        assert_eq!(config.ssl_key, None);
    }

    #[test]
    fn test_load_from_etc_only() {
        let env_setup = TestEnv::new();
        // Create a config file only in the simulated /etc/rucho directory.
        env_setup.create_config_file(
            &env_setup.etc_rucho_conf_path,
            "prefix = /etc/path\nlog_level = etc_level",
        );

        // Specify a non-existent path for the CWD config to ensure it's not loaded.
        let non_existent_cwd_conf = env_setup
            .cwd_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent.conf");

        let config = Config::load_from_paths(
            Some(env_setup.etc_rucho_conf_path.clone()),
            Some(non_existent_cwd_conf),
        );

        // Assert that values from /etc/rucho/rucho.conf are loaded.
        assert_eq!(config.prefix, "/etc/path");
        assert_eq!(config.log_level, "etc_level");
        // Assert that other values remain default.
        assert_eq!(config.server_listen_primary, "0.0.0.0:8080");
    }

    #[test]
    fn test_load_from_cwd_only() {
        let env_setup = TestEnv::new();
        // Create a config file only in the simulated CWD.
        env_setup.create_config_file(
            &env_setup.cwd_rucho_conf_path,
            "prefix = /cwd/path\nlog_level = cwd_level",
        );

        // Specify a non-existent path for the /etc config.
        let non_existent_etc_conf = env_setup
            .etc_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent.conf");

        let config = Config::load_from_paths(
            Some(non_existent_etc_conf),
            Some(env_setup.cwd_rucho_conf_path.clone()),
        );

        // Assert that values from ./rucho.conf are loaded.
        assert_eq!(config.prefix, "/cwd/path");
        assert_eq!(config.log_level, "cwd_level");
        // Assert that other values remain default.
        assert_eq!(config.server_listen_secondary, "0.0.0.0:9090");
    }

    #[test]
    fn test_cwd_overrides_etc() {
        let env_setup = TestEnv::new();
        // Create config files in both /etc and CWD with some overlapping keys.
        env_setup.create_config_file(
            &env_setup.etc_rucho_conf_path,
            "prefix = /etc/path\nlog_level = etc_level",
        );
        env_setup.create_config_file(
            &env_setup.cwd_rucho_conf_path,
            "prefix = /cwd/path\nserver_listen_primary = 1.1.1.1:1111",
        );

        let config = Config::load_from_paths(
            Some(env_setup.etc_rucho_conf_path.clone()),
            Some(env_setup.cwd_rucho_conf_path.clone()),
        );

        // Assert that CWD values override /etc values for overlapping keys.
        assert_eq!(config.prefix, "/cwd/path"); // CWD prefix wins.
                                                // Assert that non-overlapping keys are merged.
        assert_eq!(config.log_level, "etc_level"); // From /etc, as not in CWD file.
        assert_eq!(config.server_listen_primary, "1.1.1.1:1111"); // From CWD.
                                                                  // Assert that other values remain default.
        assert_eq!(config.server_listen_secondary, "0.0.0.0:9090");
    }

    #[test]
    fn test_env_overrides_all_files() {
        let env_setup = TestEnv::new();
        // Create config files in both /etc and CWD.
        env_setup.create_config_file(
            &env_setup.etc_rucho_conf_path,
            "prefix = /etc/path\nlog_level = etc_level",
        );
        env_setup.create_config_file(
            &env_setup.cwd_rucho_conf_path,
            "prefix = /cwd/path\nlog_level = cwd_level",
        );

        // Set environment variables that should override file configurations.
        env::set_var("RUCHO_PREFIX", "/env/path");
        env::set_var("RUCHO_LOG_LEVEL", "env_level");
        env::set_var("RUCHO_SERVER_LISTEN_PRIMARY", "env_primary");

        let config = Config::load_from_paths(
            Some(env_setup.etc_rucho_conf_path.clone()),
            Some(env_setup.cwd_rucho_conf_path.clone()),
        );

        // Assert that environment variable values override all file-based values.
        assert_eq!(config.prefix, "/env/path");
        assert_eq!(config.log_level, "env_level");
        assert_eq!(config.server_listen_primary, "env_primary");
        // Assert that other values remain default.
        assert_eq!(config.server_listen_secondary, "0.0.0.0:9090");
    }

    #[test]
    fn test_partial_configs_layering() {
        let env_setup = TestEnv::new();
        // /etc/rucho/rucho.conf sets prefix and an initial log_level.
        env_setup.create_config_file(
            &env_setup.etc_rucho_conf_path,
            "prefix = /etc/path\nlog_level = etc_level_original",
        );
        // ./rucho.conf overrides log_level and sets server_listen_secondary.
        env_setup.create_config_file(
            &env_setup.cwd_rucho_conf_path,
            "log_level = cwd_level\nserver_listen_secondary = 2.2.2.2:2222",
        );

        // Environment variable sets server_listen_primary.
        env::set_var("RUCHO_SERVER_LISTEN_PRIMARY", "env_primary");
        // Ensure other relevant env vars are not set from previous tests for this specific test,
        // so we only test the intended layering.
        env::remove_var("RUCHO_PREFIX");
        env::remove_var("RUCHO_LOG_LEVEL");

        let config = Config::load_from_paths(
            Some(env_setup.etc_rucho_conf_path.clone()),
            Some(env_setup.cwd_rucho_conf_path.clone()),
        );

        // prefix should come from /etc/rucho/rucho.conf
        assert_eq!(config.prefix, "/etc/path");
        // log_level should be from ./rucho.conf (overriding /etc)
        assert_eq!(config.log_level, "cwd_level");
        // server_listen_primary should be from environment variable
        assert_eq!(config.server_listen_primary, "env_primary");
        // server_listen_secondary should be from ./rucho.conf
        assert_eq!(config.server_listen_secondary, "2.2.2.2:2222");
    }

    #[test]
    fn test_load_ssl_from_file() {
        let env_setup = TestEnv::new();
        env_setup.create_config_file(
            &env_setup.cwd_rucho_conf_path,
            "ssl_cert = /test/cert.pem\nssl_key = /test/key.pem",
        );

        // For etc, pass a path that won't exist
        let non_existent_etc = env_setup
            .etc_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent.conf");

        let config = Config::load_from_paths(
            Some(non_existent_etc),
            Some(env_setup.cwd_rucho_conf_path.clone()),
        );

        assert_eq!(config.ssl_cert, Some("/test/cert.pem".to_string()));
        assert_eq!(config.ssl_key, Some("/test/key.pem".to_string()));
    }

    #[test]
    fn test_load_ssl_from_env() {
        let env_setup = TestEnv::new();
        env::set_var("RUCHO_SSL_CERT", "/env/cert.pem");
        env::set_var("RUCHO_SSL_KEY", "/env/key.pem");

        // Pass non-existent paths for files to ensure only env vars are loaded
        let non_existent_etc = env_setup
            .etc_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent_env_only_etc.conf");
        let non_existent_cwd = env_setup
            .cwd_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent_env_only_cwd.conf");

        let config = Config::load_from_paths(Some(non_existent_etc), Some(non_existent_cwd));

        assert_eq!(config.ssl_cert, Some("/env/cert.pem".to_string()));
        assert_eq!(config.ssl_key, Some("/env/key.pem".to_string()));
    }

    #[test]
    fn test_env_overrides_file_for_ssl() {
        let env_setup = TestEnv::new();
        env_setup.create_config_file(
            &env_setup.cwd_rucho_conf_path,
            "ssl_cert = /file/cert.pem\nssl_key = /file/key.pem",
        );
        env::set_var("RUCHO_SSL_CERT", "/env/cert.pem");
        env::set_var("RUCHO_SSL_KEY", "/env/key.pem");

        // For etc, pass a path that won't exist
        let non_existent_etc = env_setup
            .etc_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent.conf");

        let config = Config::load_from_paths(
            Some(non_existent_etc),
            Some(env_setup.cwd_rucho_conf_path.clone()),
        );

        assert_eq!(config.ssl_cert, Some("/env/cert.pem".to_string()));
        assert_eq!(config.ssl_key, Some("/env/key.pem".to_string()));
    }

    #[test]
    fn test_partial_ssl_config_layering() {
        let env_setup = TestEnv::new();
        env_setup.create_config_file(&env_setup.cwd_rucho_conf_path, "ssl_cert = /file/cert.pem");
        env::set_var("RUCHO_SSL_KEY", "/env/key.pem");
        // Ensure RUCHO_SSL_CERT is not set from other tests for this specific test case
        env::remove_var("RUCHO_SSL_CERT");

        // For etc, pass a path that won't exist
        let non_existent_etc = env_setup
            .etc_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent.conf");

        let config = Config::load_from_paths(
            Some(non_existent_etc),
            Some(env_setup.cwd_rucho_conf_path.clone()),
        );

        assert_eq!(config.ssl_cert, Some("/file/cert.pem".to_string()));
        assert_eq!(config.ssl_key, Some("/env/key.pem".to_string()));
    }

    #[test]
    fn test_load_tcp_udp_from_file() {
        let env_setup = TestEnv::new();
        env_setup.create_config_file(
            &env_setup.cwd_rucho_conf_path,
            "server_listen_tcp = 127.0.0.1:1234\nserver_listen_udp = 127.0.0.1:5678",
        );

        let non_existent_etc = env_setup
            .etc_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent.conf");
        let config = Config::load_from_paths(
            Some(non_existent_etc),
            Some(env_setup.cwd_rucho_conf_path.clone()),
        );

        assert_eq!(config.server_listen_tcp, Some("127.0.0.1:1234".to_string()));
        assert_eq!(config.server_listen_udp, Some("127.0.0.1:5678".to_string()));
    }

    #[test]
    fn test_load_tcp_udp_from_env() {
        let env_setup = TestEnv::new();
        env::set_var("RUCHO_SERVER_LISTEN_TCP", "127.0.0.1:1234");
        env::set_var("RUCHO_SERVER_LISTEN_UDP", "127.0.0.1:5678");

        let non_existent_etc = env_setup
            .etc_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent_env_only_etc.conf");
        let non_existent_cwd = env_setup
            .cwd_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent_env_only_cwd.conf");
        let config = Config::load_from_paths(Some(non_existent_etc), Some(non_existent_cwd));

        assert_eq!(config.server_listen_tcp, Some("127.0.0.1:1234".to_string()));
        assert_eq!(config.server_listen_udp, Some("127.0.0.1:5678".to_string()));
    }

    #[test]
    fn test_env_overrides_file_for_tcp_udp() {
        let env_setup = TestEnv::new();
        env_setup.create_config_file(
            &env_setup.cwd_rucho_conf_path,
            "server_listen_tcp = /file/tcp\nserver_listen_udp = /file/udp",
        );
        env::set_var("RUCHO_SERVER_LISTEN_TCP", "/env/tcp");
        env::set_var("RUCHO_SERVER_LISTEN_UDP", "/env/udp");

        let non_existent_etc = env_setup
            .etc_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent.conf");
        let config = Config::load_from_paths(
            Some(non_existent_etc),
            Some(env_setup.cwd_rucho_conf_path.clone()),
        );

        assert_eq!(config.server_listen_tcp, Some("/env/tcp".to_string()));
        assert_eq!(config.server_listen_udp, Some("/env/udp".to_string()));
    }

    #[test]
    fn test_partial_tcp_udp_config_layering() {
        let env_setup = TestEnv::new();
        env_setup.create_config_file(
            &env_setup.cwd_rucho_conf_path,
            "server_listen_tcp = /file/tcp",
        );
        env::set_var("RUCHO_SERVER_LISTEN_UDP", "/env/udp");
        env::remove_var("RUCHO_SERVER_LISTEN_TCP");

        let non_existent_etc = env_setup
            .etc_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent.conf");
        let config = Config::load_from_paths(
            Some(non_existent_etc),
            Some(env_setup.cwd_rucho_conf_path.clone()),
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
        let _env = TestEnv::new();
        let non_existent_etc = PathBuf::from("/tmp/non_existent_compression_test_etc.conf");
        let non_existent_cwd = PathBuf::from("./non_existent_compression_test_cwd.conf");
        let config = Config::load_from_paths(Some(non_existent_etc), Some(non_existent_cwd));

        assert!(!config.compression_enabled);
    }

    #[test]
    fn test_load_compression_enabled_from_file() {
        let env_setup = TestEnv::new();
        env_setup.create_config_file(&env_setup.cwd_rucho_conf_path, "compression_enabled = true");

        let non_existent_etc = env_setup
            .etc_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent.conf");
        let config = Config::load_from_paths(
            Some(non_existent_etc),
            Some(env_setup.cwd_rucho_conf_path.clone()),
        );

        assert!(config.compression_enabled);
    }

    #[test]
    fn test_load_compression_enabled_from_env() {
        let env_setup = TestEnv::new();
        env::set_var("RUCHO_COMPRESSION_ENABLED", "true");

        let non_existent_etc = env_setup
            .etc_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent.conf");
        let non_existent_cwd = env_setup
            .cwd_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent.conf");
        let config = Config::load_from_paths(Some(non_existent_etc), Some(non_existent_cwd));

        assert!(config.compression_enabled);
    }

    #[test]
    fn test_env_overrides_file_for_compression() {
        let env_setup = TestEnv::new();
        env_setup.create_config_file(
            &env_setup.cwd_rucho_conf_path,
            "compression_enabled = false",
        );
        env::set_var("RUCHO_COMPRESSION_ENABLED", "true");

        let non_existent_etc = env_setup
            .etc_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent.conf");
        let config = Config::load_from_paths(
            Some(non_existent_etc),
            Some(env_setup.cwd_rucho_conf_path.clone()),
        );

        assert!(config.compression_enabled);
    }

    // --- Chaos config tests ---

    #[test]
    fn test_chaos_disabled_by_default() {
        let _env = TestEnv::new();
        let non_existent_etc = PathBuf::from("/tmp/non_existent_chaos_default_etc.conf");
        let non_existent_cwd = PathBuf::from("./non_existent_chaos_default_cwd.conf");
        let config = Config::load_from_paths(Some(non_existent_etc), Some(non_existent_cwd));

        assert!(!config.chaos.is_enabled());
        assert!(config.chaos.modes.is_empty());
        assert!(config.chaos.inform_header); // default true
    }

    #[test]
    fn test_chaos_config_from_file() {
        let env_setup = TestEnv::new();
        env_setup.create_config_file(
            &env_setup.cwd_rucho_conf_path,
            "chaos_mode = failure,delay\n\
             chaos_failure_rate = 0.5\n\
             chaos_failure_codes = 500,503\n\
             chaos_delay_rate = 0.3\n\
             chaos_delay_ms = 2000\n\
             chaos_inform_header = false",
        );

        let non_existent_etc = env_setup
            .etc_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent.conf");
        let config = Config::load_from_paths(
            Some(non_existent_etc),
            Some(env_setup.cwd_rucho_conf_path.clone()),
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
        let env_setup = TestEnv::new();
        env::set_var("RUCHO_CHAOS_MODE", "corruption");
        env::set_var("RUCHO_CHAOS_CORRUPTION_RATE", "0.1");
        env::set_var("RUCHO_CHAOS_CORRUPTION_TYPE", "empty");

        let non_existent_etc = env_setup
            .etc_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent.conf");
        let non_existent_cwd = env_setup
            .cwd_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent.conf");
        let config = Config::load_from_paths(Some(non_existent_etc), Some(non_existent_cwd));

        assert!(config.chaos.is_enabled());
        assert!(config.chaos.has_corruption());
        assert!((config.chaos.corruption_rate - 0.1).abs() < f64::EPSILON);
        assert_eq!(config.chaos.corruption_type, "empty");
    }

    #[test]
    fn test_chaos_env_overrides_file() {
        let env_setup = TestEnv::new();
        env_setup.create_config_file(
            &env_setup.cwd_rucho_conf_path,
            "chaos_mode = failure\n\
             chaos_failure_rate = 0.5\n\
             chaos_failure_codes = 500",
        );
        env::set_var("RUCHO_CHAOS_FAILURE_RATE", "0.9");

        let non_existent_etc = env_setup
            .etc_rucho_conf_path
            .parent()
            .unwrap()
            .join("non_existent.conf");
        let config = Config::load_from_paths(
            Some(non_existent_etc),
            Some(env_setup.cwd_rucho_conf_path.clone()),
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
}
