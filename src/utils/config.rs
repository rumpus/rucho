use std::env;
use std::fs;
use std::path::Path;

/// Holds the application configuration.
///
/// Configuration values are loaded in the following order of precedence (lowest to highest):
/// 1. Default values.
/// 2. Values from the `rucho.conf` file (if it exists in the current directory).
/// 3. Environment variables prefixed with `RUCHO_` (e.g., `RUCHO_PREFIX`).
#[derive(Debug, Clone)]
pub struct Config {
    pub prefix: String,
    pub log_level: String,
    pub server_listen_primary: String,
    pub server_listen_secondary: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            prefix: "/usr/local/rucho".to_string(),
            log_level: "notice".to_string(),
            server_listen_primary: "0.0.0.0:8080".to_string(),
            server_listen_secondary: "0.0.0.0:9090".to_string(),
        }
    }
}

impl Config {
    /// Loads the configuration for the application.
    ///
    /// It first sets default values, then attempts to read from a `rucho.conf`
    /// file in the current working directory. Finally, it overrides any values
    /// with environment variables prefixed with `RUCHO_`.
    ///
    /// The `rucho.conf` file should contain `key = value` pairs, one per line.
    /// Lines starting with `#` are treated as comments and ignored.
    /// Empty lines are also ignored.
    ///
    /// Supported keys in `rucho.conf`:
    /// - `prefix`
    /// - `log_level`
    /// - `server_listen_primary`
    /// - `server_listen_secondary`
    ///
    /// Corresponding environment variables:
    /// - `RUCHO_PREFIX`
    /// - `RUCHO_LOG_LEVEL`
    /// - `RUCHO_SERVER_LISTEN_PRIMARY`
    /// - `RUCHO_SERVER_LISTEN_SECONDARY`
    pub fn load_config() -> Self {
        let mut config = Config::default();

        // Load from rucho.conf
        if Path::new("rucho.conf").exists() {
            if let Ok(contents) = fs::read_to_string("rucho.conf") {
                for line in contents.lines() {
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
                            _ => eprintln!("Warning: Unknown key in rucho.conf: {}", key),
                        }
                    } else {
                        eprintln!("Warning: Invalid line in rucho.conf: {}", line);
                    }
                }
            } else {
                eprintln!("Warning: Could not read rucho.conf");
            }
        }

        // Override with environment variables
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

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::io::Write;

    // Helper to create a temporary config file
    fn create_temp_config_file(name: &str, content: &str) {
        let mut file = fs::File::create(name).unwrap();
        writeln!(file, "{}", content).unwrap();
    }

    // Helper to clean up temp file and env vars
    fn cleanup_test_environment(file_name: Option<&str>) {
        if let Some(name) = file_name {
            let _ = fs::remove_file(name);
        }
        env::remove_var("RUCHO_PREFIX");
        env::remove_var("RUCHO_LOG_LEVEL");
        env::remove_var("RUCHO_SERVER_LISTEN_PRIMARY");
        env::remove_var("RUCHO_SERVER_LISTEN_SECONDARY");
    }

    #[test]
    fn test_default_config() {
        cleanup_test_environment(None);
        let config = Config::load_config();
        assert_eq!(config.prefix, "/usr/local/rucho");
        assert_eq!(config.log_level, "notice");
        assert_eq!(config.server_listen_primary, "0.0.0.0:8080");
        assert_eq!(config.server_listen_secondary, "0.0.0.0:9090");
    }

    #[test]
    fn test_rucho_conf_overrides_defaults() {
        let conf_file_name = "rucho.conf.test_file_override";
        let content = "prefix = /tmp/rucho
log_level = debug";
        create_temp_config_file(conf_file_name, content);
        
        // Temporarily rename to rucho.conf for the test
        fs::rename(conf_file_name, "rucho.conf").unwrap();

        cleanup_test_environment(None); // Clean env vars, keep file for now

        let config = Config::load_config();
        assert_eq!(config.prefix, "/tmp/rucho");
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.server_listen_primary, "0.0.0.0:8080"); // Default
        assert_eq!(config.server_listen_secondary, "0.0.0.0:9090"); // Default
        
        cleanup_test_environment(Some("rucho.conf")); // Clean up the renamed file
    }

    #[test]
    fn test_env_vars_override_all() {
        let conf_file_name = "rucho.conf.test_env_override";
        let content = "prefix = /file/prefix
log_level = file_level";
        create_temp_config_file(conf_file_name, content);
        fs::rename(conf_file_name, "rucho.conf").unwrap();

        env::set_var("RUCHO_PREFIX", "/env/prefix");
        env::set_var("RUCHO_LOG_LEVEL", "env_level");
        env::set_var("RUCHO_SERVER_LISTEN_PRIMARY", "127.0.0.1:1111");
        env::set_var("RUCHO_SERVER_LISTEN_SECONDARY", "127.0.0.1:2222");

        let config = Config::load_config();
        assert_eq!(config.prefix, "/env/prefix");
        assert_eq!(config.log_level, "env_level");
        assert_eq!(config.server_listen_primary, "127.0.0.1:1111");
        assert_eq!(config.server_listen_secondary, "127.0.0.1:2222");
        
        cleanup_test_environment(Some("rucho.conf"));
    }

    #[test]
    fn test_mixed_config_sources() {
        let conf_file_name = "rucho.conf.test_mixed";
        // Only prefix and primary_listen are in the file
        let content = "prefix = /file/prefix
server_listen_primary = 0.0.0.0:7777"; 
        create_temp_config_file(conf_file_name, content);
        fs::rename(conf_file_name, "rucho.conf").unwrap();

        // Env vars override prefix and set log_level
        env::set_var("RUCHO_PREFIX", "/env/prefix"); // Overrides file
        env::set_var("RUCHO_LOG_LEVEL", "env_debug"); // No file equivalent, overrides default

        let config = Config::load_config();
        assert_eq!(config.prefix, "/env/prefix"); // Env overrides file
        assert_eq!(config.log_level, "env_debug"); // Env overrides default
        assert_eq!(config.server_listen_primary, "0.0.0.0:7777"); // From file
        assert_eq!(config.server_listen_secondary, "0.0.0.0:9090"); // Default
        
        cleanup_test_environment(Some("rucho.conf"));
    }

    #[test]
    fn test_invalid_lines_in_config_file() {
        let conf_file_name = "rucho.conf.test_invalid";
        let content = "prefix = /valid/prefix
thisisinvalid
log_level = valid_level
=novaluekey";
        create_temp_config_file(conf_file_name, content);
        fs::rename(conf_file_name, "rucho.conf").unwrap();
        
        // We can't directly assert eprintln output here without more complex test setup.
        // So, we'll just verify that valid entries are still loaded.
        // The warnings for invalid lines would be manually checked during development or CI.
        let config = Config::load_config();
        assert_eq!(config.prefix, "/valid/prefix");
        assert_eq!(config.log_level, "valid_level"); // Falls back to default as "valid_level" is not set by env
        
        cleanup_test_environment(Some("rucho.conf"));
    }

     #[test]
    fn test_empty_config_file() {
        let conf_file_name = "rucho.conf.test_empty";
        create_temp_config_file(conf_file_name, ""); // Empty content
        fs::rename(conf_file_name, "rucho.conf").unwrap();

        cleanup_test_environment(None); // Clean env vars

        let config = Config::load_config();
        // Should load all defaults
        assert_eq!(config.prefix, "/usr/local/rucho");
        assert_eq!(config.log_level, "notice");
        assert_eq!(config.server_listen_primary, "0.0.0.0:8080");
        assert_eq!(config.server_listen_secondary, "0.0.0.0:9090");

        cleanup_test_environment(Some("rucho.conf"));
    }

    #[test]
    fn test_config_file_with_comments_and_whitespace() {
        let conf_file_name = "rucho.conf.test_comments";
        let content = "# This is a comment
   prefix = /commented/prefix   

log_level=spaced_level
#another comment";
        create_temp_config_file(conf_file_name, content);
        fs::rename(conf_file_name, "rucho.conf").unwrap();
        
        cleanup_test_environment(None);

        let config = Config::load_config();
        assert_eq!(config.prefix, "/commented/prefix");
        assert_eq!(config.log_level, "spaced_level"); //This should be "spaced_level"
        
        cleanup_test_environment(Some("rucho.conf"));
    }
}
