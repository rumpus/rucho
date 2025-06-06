use std::env;
use std::fs;
use std::path::PathBuf; // Modified to remove unused Path

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
    /// Optional path to an SSL certificate file for HTTPS. Required if any listen address uses "ssl:".
    pub ssl_cert: Option<String>,
    /// Optional path to an SSL private key file for HTTPS. Required if any listen address uses "ssl:".
    pub ssl_key: Option<String>,
}

impl Default for Config {
    /// Provides the hardcoded default configuration values for the application.
    /// These defaults are the first layer in the configuration loading process.
    fn default() -> Self {
        Config {
            prefix: "/usr/local/rucho".to_string(),
            log_level: "info".to_string(),
            server_listen_primary: "0.0.0.0:8080".to_string(),
            server_listen_secondary: "0.0.0.0:9090".to_string(),
            ssl_cert: None,
            ssl_key: None,
        }
    }
}

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
                    "ssl_cert" => config.ssl_cert = Some(value.to_string()),
                    "ssl_key" => config.ssl_key = Some(value.to_string()),
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
    fn load_from_paths(etc_path_override: Option<PathBuf>, local_path_override: Option<PathBuf>) -> Self {
        let mut config = Config::default();

        // Determine paths to use: override or default.
        let etc_config_path = etc_path_override.unwrap_or_else(|| PathBuf::from("/etc/rucho/rucho.conf"));
        let local_config_path = local_path_override.unwrap_or_else(|| PathBuf::from("rucho.conf"));

        // Load from the system-wide config file (e.g., /etc/rucho/rucho.conf or override)
        if etc_config_path.exists() {
            if let Ok(contents) = fs::read_to_string(&etc_config_path) {
                Self::parse_file_contents(&mut config, contents);
            } else {
                // Log a warning if the file exists but cannot be read
                eprintln!("Warning: Could not read system config file at {:?}, though it exists.", etc_config_path);
            }
        }

        // Load from the local config file (e.g., ./rucho.conf or override), overriding previous values
        if local_config_path.exists() {
            if let Ok(contents) = fs::read_to_string(&local_config_path) {
                Self::parse_file_contents(&mut config, contents);
            } else {
                // Log a warning if the file exists but cannot be read
                eprintln!("Warning: Could not read local config file at {:?}, though it exists.", local_config_path);
            }
        }

        // Override with environment variables (RUCHO_ prefixed)
        if let Ok(prefix) = env::var("RUCHO_PREFIX") {
            config.prefix = prefix;
        }
        if let Ok(log_level) = env::var("RUCHO_LOG_LEVEL") {
            config.log_level = log_level;
        }
        if let Ok(server_listen_primary) = env::var("RUCHO_SERVER_LISTEN_PRIMARY") {
            config.server_listen_primary = server_listen_primary;
        }
        if let Ok(server_listen_secondary) = env::var("RUCHO_SERVER_LISTEN_SECONDARY") {
            config.server_listen_secondary = server_listen_secondary;
        }
        if let Ok(ssl_cert) = env::var("RUCHO_SSL_CERT") {
            config.ssl_cert = Some(ssl_cert);
        }
        if let Ok(ssl_key) = env::var("RUCHO_SSL_KEY") {
            config.ssl_key = Some(ssl_key);
        }

        config
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
    pub fn load() -> Self {
        Self::load_from_paths(None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::{self, File}; // Added File
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir; // Used for creating temporary directories for config files.

    // Module for configuration loading tests.
    // TestEnv struct helps manage temporary directories and files for testing config loading.
    /// Sets up a controlled environment for testing configuration loading.
    ///
    /// This involves creating temporary directories to simulate `/etc/rucho/` and a
    /// temporary current working directory (CWD). It also handles cleaning up
    /// environment variables and restoring the original CWD when dropped.
    struct TestEnv {
        _etc_dir: TempDir, // Temporary directory for simulating /etc
        etc_rucho_conf_path: PathBuf, // Path to the simulated /etc/rucho/rucho.conf
        _cwd_dir: TempDir, // Temporary directory for simulating the CWD
        cwd_rucho_conf_path: PathBuf, // Path to the simulated ./rucho.conf in the temp CWD
        original_cwd: PathBuf, // Stores the original CWD to restore it later
    }

    impl TestEnv {
        /// Creates a new `TestEnv` instance.
        ///
        /// This sets up temporary directories for `/etc/rucho` and a test-specific CWD.
        /// It also changes the current directory to the temporary CWD.
        fn new() -> Self {
            // Create a temporary directory to simulate /etc
            let etc_dir = TempDir::new().expect("Failed to create temp etc dir");
            let etc_rucho_dir = etc_dir.path().join("rucho"); // Simulate /etc/rucho
            fs::create_dir_all(&etc_rucho_dir).expect("Failed to create fake /etc/rucho");
            
            // Create a temporary directory to act as the CWD for the test
            let cwd_dir = TempDir::new().expect("Failed to create temp cwd dir");
            
            // Store original CWD and change to the temporary CWD
            let original_cwd = env::current_dir().unwrap();
            env::set_current_dir(cwd_dir.path()).unwrap();

            TestEnv {
                etc_rucho_conf_path: etc_rucho_dir.join("rucho.conf"),
                _etc_dir: etc_dir, // Keep TempDir to ensure it's cleaned up on drop
                cwd_rucho_conf_path: cwd_dir.path().join("rucho.conf"), // Path to ./rucho.conf in temp CWD
                _cwd_dir: cwd_dir, // Keep TempDir for CWD
                original_cwd,
            }
        }

        /// Helper function to create a configuration file with specified content at a given path.
        fn create_config_file(&self, path: &std::path::Path, content: &str) {
            let mut file = File::create(path).unwrap_or_else(|e| {
                panic!("Failed to create config file at {:?}: {}", path, e) // Panic if file creation fails
            });
            writeln!(file, "{}", content).unwrap(); // Write content to the file
        }
    }

    /// Restores the original CWD and cleans up environment variables set during tests.
    impl Drop for TestEnv {
        fn drop(&mut self) {
            // Restore the original current working directory
            env::set_current_dir(&self.original_cwd).unwrap();
            // TempDirs (_etc_dir, _cwd_dir) are automatically removed when they go out of scope.
            // Clean up any environment variables that might have been set by tests.
            env::remove_var("RUCHO_PREFIX");
            env::remove_var("RUCHO_LOG_LEVEL");
            env::remove_var("RUCHO_SERVER_LISTEN_PRIMARY");
            env::remove_var("RUCHO_SERVER_LISTEN_SECONDARY");
            env::remove_var("RUCHO_SSL_CERT");
            env::remove_var("RUCHO_SSL_KEY");
        }
    }
    
    #[test]
    fn test_default_config() {
        let _env = TestEnv::new(); // Sets up CWD, cleans up vars.
        // To test defaults, we call load_from_paths with paths that are guaranteed not to exist.
        // This ensures that only the hardcoded defaults are loaded.
        let non_existent_etc = PathBuf::from("/tmp/non_existent_rucho_config_for_default_test_etc.conf");
        let non_existent_cwd = PathBuf::from("./non_existent_rucho_config_for_default_test_cwd.conf");
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
        env_setup.create_config_file(&env_setup.etc_rucho_conf_path, "prefix = /etc/path\nlog_level = etc_level");

        // Specify a non-existent path for the CWD config to ensure it's not loaded.
        let non_existent_cwd_conf = env_setup.cwd_rucho_conf_path.parent().unwrap().join("non_existent.conf");

        let config = Config::load_from_paths(Some(env_setup.etc_rucho_conf_path.clone()), Some(non_existent_cwd_conf));
        
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
        env_setup.create_config_file(&env_setup.cwd_rucho_conf_path, "prefix = /cwd/path\nlog_level = cwd_level");

        // Specify a non-existent path for the /etc config.
        let non_existent_etc_conf = env_setup.etc_rucho_conf_path.parent().unwrap().join("non_existent.conf");

        let config = Config::load_from_paths(Some(non_existent_etc_conf), Some(env_setup.cwd_rucho_conf_path.clone()));

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
        env_setup.create_config_file(&env_setup.etc_rucho_conf_path, "prefix = /etc/path\nlog_level = etc_level");
        env_setup.create_config_file(&env_setup.cwd_rucho_conf_path, "prefix = /cwd/path\nserver_listen_primary = 1.1.1.1:1111");

        let config = Config::load_from_paths(Some(env_setup.etc_rucho_conf_path.clone()), Some(env_setup.cwd_rucho_conf_path.clone()));

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
        env_setup.create_config_file(&env_setup.etc_rucho_conf_path, "prefix = /etc/path\nlog_level = etc_level");
        env_setup.create_config_file(&env_setup.cwd_rucho_conf_path, "prefix = /cwd/path\nlog_level = cwd_level");

        // Set environment variables that should override file configurations.
        env::set_var("RUCHO_PREFIX", "/env/path");
        env::set_var("RUCHO_LOG_LEVEL", "env_level");
        env::set_var("RUCHO_SERVER_LISTEN_PRIMARY", "env_primary");

        let config = Config::load_from_paths(Some(env_setup.etc_rucho_conf_path.clone()), Some(env_setup.cwd_rucho_conf_path.clone()));

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
        env_setup.create_config_file(&env_setup.etc_rucho_conf_path, "prefix = /etc/path\nlog_level = etc_level_original");
        // ./rucho.conf overrides log_level and sets server_listen_secondary.
        env_setup.create_config_file(&env_setup.cwd_rucho_conf_path, "log_level = cwd_level\nserver_listen_secondary = 2.2.2.2:2222");

        // Environment variable sets server_listen_primary.
        env::set_var("RUCHO_SERVER_LISTEN_PRIMARY", "env_primary");
        // Ensure other relevant env vars are not set from previous tests for this specific test,
        // so we only test the intended layering.
        env::remove_var("RUCHO_PREFIX"); 
        env::remove_var("RUCHO_LOG_LEVEL");


        let config = Config::load_from_paths(Some(env_setup.etc_rucho_conf_path.clone()), Some(env_setup.cwd_rucho_conf_path.clone()));
        
        // prefix should come from /etc/rucho/rucho.conf
        assert_eq!(config.prefix, "/etc/path");
        // log_level should be from ./rucho.conf (overriding /etc)
        assert_eq!(config.log_level, "cwd_level");
        // server_listen_primary should be from environment variable
        assert_eq!(config.server_listen_primary, "env_primary");
        // server_listen_secondary should be from ./rucho.conf
        assert_eq!(config.server_listen_secondary, "2.2.2.2:2222");
    }

    // Test for ensuring pub fn load() calls load_from_paths(None, None).
    // This test is a bit conceptual as it's hard to directly verify without more complex mocking
    // or by observing side effects if load_from_paths had them (which it doesn't beyond eprintln).
    // For now, we trust the implementation. A more advanced setup might involve:
    // 1. Feature flags to compile a version of load_from_paths that logs its inputs.
    // 2. Using a mocking library if one were available and suitable.
    // Given the current tools, a direct test of load() calling load_from_paths(None,None)
    // is hard to achieve perfectly. We assume the code structure is correct.
    // The existing test_default_config implicitly tests this if no actual default files exist.
}

#[test]
fn test_load_ssl_from_file() {
    let env_setup = TestEnv::new();
    env_setup.create_config_file(
        &env_setup.cwd_rucho_conf_path,
        "ssl_cert = /test/cert.pem\nssl_key = /test/key.pem",
    );

    // For etc, pass a path that won't exist
    let non_existent_etc = env_setup.etc_rucho_conf_path.parent().unwrap().join("non_existent.conf");

    let config = Config::load_from_paths(Some(non_existent_etc), Some(env_setup.cwd_rucho_conf_path.clone()));

    assert_eq!(config.ssl_cert, Some("/test/cert.pem".to_string()));
    assert_eq!(config.ssl_key, Some("/test/key.pem".to_string()));
}

#[test]
fn test_load_ssl_from_env() {
    let env_setup = TestEnv::new(); // Use TestEnv to ensure CWD is set and vars are cleaned up
    env::set_var("RUCHO_SSL_CERT", "/env/cert.pem");
    env::set_var("RUCHO_SSL_KEY", "/env/key.pem");

    // Pass non-existent paths for files to ensure only env vars are loaded
    let non_existent_etc = env_setup.etc_rucho_conf_path.parent().unwrap().join("non_existent_env_only_etc.conf");
    let non_existent_cwd = env_setup.cwd_rucho_conf_path.parent().unwrap().join("non_existent_env_only_cwd.conf");


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
    let non_existent_etc = env_setup.etc_rucho_conf_path.parent().unwrap().join("non_existent.conf");

    let config = Config::load_from_paths(Some(non_existent_etc), Some(env_setup.cwd_rucho_conf_path.clone()));

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
    let non_existent_etc = env_setup.etc_rucho_conf_path.parent().unwrap().join("non_existent.conf");

    let config = Config::load_from_paths(Some(non_existent_etc), Some(env_setup.cwd_rucho_conf_path.clone()));

    assert_eq!(config.ssl_cert, Some("/file/cert.pem".to_string()));
    assert_eq!(config.ssl_key, Some("/env/key.pem".to_string()));
}
