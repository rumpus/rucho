// src/utils/server_config.rs

// This module configures the server to use optional HTTPS + HTTP/2 using rustls.
// If the certificates are not found, it falls back to plain HTTP with HTTP/1.1.

use std::path::PathBuf;
use axum_server::tls_rustls::RustlsConfig;

/// Attempts to load Rustls configuration for enabling HTTPS.
///
/// This function checks for the existence of SSL certificate and key files at the
/// paths provided. If both files are found and valid, it returns a `RustlsConfig`
/// suitable for configuring an Axum server with TLS.
///
/// If either path is not provided, or if the files are not found or are invalid,
/// this function logs a warning/error and returns `None`, indicating that TLS
/// should not be enabled.
///
/// # Arguments
///
/// * `ssl_cert_path_opt`: An `Option<&str>` containing the path to the SSL certificate file.
/// * `ssl_key_path_opt`: An `Option<&str>` containing the path to the SSL private key file.
///
/// # Returns
///
/// An `Option<RustlsConfig>`. `Some(RustlsConfig)` if TLS can be configured, `None` otherwise.
pub async fn try_load_rustls_config(ssl_cert_path_opt: Option<&str>, ssl_key_path_opt: Option<&str>) -> Option<RustlsConfig> {
    // Check if both paths are provided
    let (cert_p, key_p) = match (ssl_cert_path_opt, ssl_key_path_opt) {
        (Some(cert_path_str), Some(key_path_str)) => (cert_path_str, key_path_str),
        _ => {
            // If either path (or both) is None, SSL cannot be configured.
            // It's up to the caller to decide if this is a warning or info.
            // For this function, we just return None as requested.
            tracing::debug!("SSL certificate or key path not provided, or only one was provided.");
            return None;
        }
    };

    let cert_path = PathBuf::from(cert_p);
    let key_path = PathBuf::from(key_p);

    // Check if both certificate and key files exist at the provided paths
    if cert_path.exists() && key_path.exists() {
        match RustlsConfig::from_pem_file(&cert_path, &key_path).await {
            Ok(config) => Some(config),
            Err(err) => {
                tracing::error!("Failed to load TLS config from {} and {}: {}", cert_path.display(), key_path.display(), err);
                None
            }
        }
    } else {
        tracing::warn!("TLS certificate or key file not found at the specified path(s): {} or {}. Cannot enable TLS.", cert_path.display(), key_path.display());
        None
    }
}

/// Parses a server listen address string to extract the address and SSL flag.
///
/// The input string can be in the format "IP:PORT" or "IP:PORT ssl".
/// If the string ends with " ssl" (case-sensitive), the SSL flag is set to true.
///
/// # Arguments
///
/// * `listen_str`: A string slice (`&str`) representing the listen address configuration.
///
/// # Returns
///
/// An `Option<(String, bool)>`.
/// - `Some((address, is_ssl))` where `address` is the IP:PORT part and `is_ssl`
///   is true if " ssl" was present.
/// - `None` if the input `listen_str` is empty.
pub fn parse_listen_address(listen_str: &str) -> Option<(String, bool)> {
    if listen_str.is_empty() {
        return None;
    }

    if listen_str.ends_with(" ssl") {
        Some((listen_str.trim_end_matches(" ssl").to_string(), true))
    } else {
        Some((listen_str.to_string(), false))
    }
}
