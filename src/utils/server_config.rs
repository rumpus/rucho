// src/utils/server_config.rs

// This module configures the server to use optional HTTPS + HTTP/2 using rustls.
// If the certificates are not found, it falls back to plain HTTP with HTTP/1.1.

use std::path::PathBuf;
use axum_server::tls_rustls::RustlsConfig;

/// Determines whether to launch with HTTPS or plain HTTP.
/// Returns either a `RustlsConfig` if certs are found, or `None` for plain HTTP.
pub async fn try_load_rustls_config() -> Option<RustlsConfig> {
    let cert_path = PathBuf::from("certs/cert.pem");
    let key_path = PathBuf::from("certs/key.pem");

    // Check if both certificate and key files exist
    if cert_path.exists() && key_path.exists() {
        match RustlsConfig::from_pem_file(&cert_path, &key_path).await {
            Ok(config) => Some(config),
            Err(err) => {
                tracing::error!("Failed to load TLS config: {err}");
                None
            }
        }
    } else {
        tracing::warn!("TLS certs not found. Running in HTTP (no TLS) mode.");
        None
    }
}
