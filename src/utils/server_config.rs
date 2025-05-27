// src/utils/server_config.rs

// This module configures the server to use optional HTTPS + HTTP/2 using rustls.
// If the certificates are not found, it falls back to plain HTTP with HTTP/1.1.

use std::path::PathBuf;
use axum_server::tls_rustls::RustlsConfig;

/// Determines whether to launch with HTTPS or plain HTTP.
/// Returns either a `RustlsConfig` if certs are found, or `None` for plain HTTP.
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

/// Parses a listen address string (e.g., "0.0.0.0:8080" or "0.0.0.0:8043 ssl").
///
/// Returns `Some((address_part, is_ssl))` if the string is valid,
/// or `None` if the input string is empty.
/// `is_ssl` is true if the string ends with " ssl".
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
