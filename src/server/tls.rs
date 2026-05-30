//! TLS connection-info echo support for the HTTPS listener.
//!
//! `axum_server::bind_rustls` performs the TLS handshake internally and never
//! surfaces the underlying [`rustls::ServerConnection`] to handlers, so echo
//! endpoints cannot normally report the negotiated TLS parameters. This module
//! closes that gap with [`TlsInfoAcceptor`], a thin wrapper around axum-server's
//! [`RustlsAcceptor`] that:
//!
//! 1. lets the inner acceptor complete the handshake (it resolves to a real
//!    [`tokio_rustls::server::TlsStream`], handshake already done — see
//!    `axum_server::tls_rustls::future::RustlsAcceptorFuture`),
//! 2. reads the negotiated version / cipher / ALPN / client certs off the
//!    `&ServerConnection` via `TlsStream::get_ref`, and
//! 3. layers a [`TlsConnectionInfo`] onto the per-connection service as a
//!    request extension so handlers can pick it up with
//!    `Option<Extension<Arc<TlsConnectionInfo>>>`.
//!
//! HTTP/2 ALPN, graceful shutdown via the `Handle`, and connect-info are all
//! preserved because we delegate the actual accept to the inner `RustlsAcceptor`
//! and only decorate its results.

use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;

use axum_server::accept::Accept;
use axum_server::tls_rustls::{RustlsAcceptor, RustlsConfig};
use rustls::ServerConnection;
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
use tower_http::add_extension::AddExtension;

/// The negotiated TLS parameters of a single HTTPS connection.
///
/// Surfaced as a request extension by [`TlsInfoAcceptor`] and echoed under the
/// `tls` key by the `/get` and `/anything` handlers. All fields are best-effort:
/// any value rustls did not negotiate is `None`/empty.
#[derive(Debug, Clone)]
pub struct TlsConnectionInfo {
    /// Negotiated protocol version, e.g. `"TLSv1.3"` / `"TLSv1.2"`.
    pub version: Option<String>,
    /// Negotiated cipher suite, e.g. `"TLS13_AES_128_GCM_SHA256"`.
    pub cipher_suite: Option<String>,
    /// Negotiated ALPN protocol, e.g. `"h2"` / `"http/1.1"`.
    pub alpn: Option<String>,
    /// Whether the client presented a certificate (only under mTLS).
    pub client_cert_present: bool,
    /// DER byte-length of each presented client certificate, leaf-first.
    /// Empty unless client-cert auth (mTLS) is configured.
    pub client_certs: Vec<usize>,
}

impl TlsConnectionInfo {
    /// Extracts the negotiated TLS parameters from an established
    /// [`rustls::ServerConnection`].
    pub fn from_server_connection(conn: &ServerConnection) -> Self {
        // rustls renders versions as e.g. "TLSv1_3"; normalize the underscore to
        // the conventional "TLSv1.3" form that curl/OpenSSL/browsers display.
        // (Cipher-suite underscores below are meaningful IANA naming — left as-is.)
        let version = conn
            .protocol_version()
            .and_then(|v| v.as_str())
            .map(|s| s.replace('_', "."));

        let cipher_suite = conn
            .negotiated_cipher_suite()
            .and_then(|cs| cs.suite().as_str())
            .map(str::to_owned);

        let alpn = conn
            .alpn_protocol()
            .map(|p| String::from_utf8_lossy(p).into_owned());

        let client_certs: Vec<usize> = conn
            .peer_certificates()
            .map(|certs| certs.iter().map(|c| c.as_ref().len()).collect())
            .unwrap_or_default();

        Self {
            version,
            cipher_suite,
            alpn,
            client_cert_present: !client_certs.is_empty(),
            client_certs,
        }
    }

    /// Renders the info as the JSON object echoed under the `tls` key.
    pub fn to_json(&self) -> Value {
        json!({
            "version": self.version,
            "cipher_suite": self.cipher_suite,
            "alpn": self.alpn,
            "client_cert_present": self.client_cert_present,
            "client_certs": self
                .client_certs
                .iter()
                .map(|len| json!({ "der_length": len }))
                .collect::<Vec<_>>(),
        })
    }
}

/// An [`Accept`] wrapper that decorates each accepted HTTPS connection with a
/// [`TlsConnectionInfo`] request extension.
///
/// Construct with [`TlsInfoAcceptor::new`] from the same [`RustlsConfig`] you
/// would pass to `axum_server::bind_rustls`, then hand it to
/// `axum_server::Server::acceptor`.
#[derive(Clone)]
pub struct TlsInfoAcceptor {
    inner: RustlsAcceptor,
}

impl TlsInfoAcceptor {
    /// Wraps the given rustls config in a TLS-info-injecting acceptor.
    pub fn new(config: RustlsConfig) -> Self {
        Self {
            inner: RustlsAcceptor::new(config),
        }
    }
}

impl<S> Accept<TcpStream, S> for TlsInfoAcceptor
where
    S: Send + 'static,
{
    type Stream = TlsStream<TcpStream>;
    type Service = AddExtension<S, Arc<TlsConnectionInfo>>;
    type Future = Pin<Box<dyn Future<Output = io::Result<(Self::Stream, Self::Service)>> + Send>>;

    fn accept(&self, stream: TcpStream, service: S) -> Self::Future {
        let inner = self.inner.clone();
        Box::pin(async move {
            // The inner RustlsAcceptor drives the handshake to completion before
            // resolving, so `tls_stream` is fully negotiated here.
            let (tls_stream, service) = inner.accept(stream, service).await?;
            let info = {
                let (_io, conn) = tls_stream.get_ref();
                Arc::new(TlsConnectionInfo::from_server_connection(conn))
            };
            Ok((tls_stream, AddExtension::new(service, info)))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_json_renders_negotiated_fields() {
        let info = TlsConnectionInfo {
            version: Some("TLSv1.3".to_string()),
            cipher_suite: Some("TLS13_AES_128_GCM_SHA256".to_string()),
            alpn: Some("h2".to_string()),
            client_cert_present: false,
            client_certs: Vec::new(),
        };

        let json = info.to_json();
        assert_eq!(json["version"], "TLSv1.3");
        assert_eq!(json["cipher_suite"], "TLS13_AES_128_GCM_SHA256");
        assert_eq!(json["alpn"], "h2");
        assert_eq!(json["client_cert_present"], false);
        assert_eq!(json["client_certs"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn to_json_nulls_absent_fields_and_lists_client_certs() {
        let info = TlsConnectionInfo {
            version: None,
            cipher_suite: None,
            alpn: None,
            client_cert_present: true,
            client_certs: vec![1200, 980],
        };

        let json = info.to_json();
        assert!(json["version"].is_null());
        assert!(json["cipher_suite"].is_null());
        assert!(json["alpn"].is_null());
        assert_eq!(json["client_cert_present"], true);
        let certs = json["client_certs"].as_array().unwrap();
        assert_eq!(certs.len(), 2);
        assert_eq!(certs[0]["der_length"], 1200);
        assert_eq!(certs[1]["der_length"], 980);
    }
}
