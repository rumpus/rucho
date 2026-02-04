//! TCP echo server setup.

use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::tcp_udp_handlers::handle_tcp_connection;

/// Sets up a TCP echo listener on the given address.
///
/// Parses the address string and binds a TCP listener. Incoming connections
/// are handled by `handle_tcp_connection` which echoes data back to clients.
pub async fn setup_tcp_listener(
    tcp_addr_str: &str,
    server_handles: &mut Vec<JoinHandle<Result<(), std::io::Error>>>,
) {
    let addr: std::net::SocketAddr = match tcp_addr_str.parse() {
        Ok(addr) => addr,
        Err(e) => {
            tracing::error!("Failed to parse TCP address '{}': {}", tcp_addr_str, e);
            return;
        }
    };

    match TcpListener::bind(addr).await {
        Ok(listener) => {
            tracing::info!("Starting TCP echo listener on {}", addr);
            let tcp_listener_handle = tokio::spawn(async move {
                loop {
                    match listener.accept().await {
                        Ok((socket, client_addr)) => {
                            tracing::info!("Accepted new TCP connection from {}", client_addr);
                            tokio::spawn(handle_tcp_connection(socket));
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to accept TCP connection: {}. Listener loop continues.",
                                e
                            );
                        }
                    }
                }
                #[allow(unreachable_code)]
                Ok::<(), std::io::Error>(())
            });
            server_handles.push(tcp_listener_handle);
        }
        Err(e) => {
            tracing::error!("Failed to bind TCP listener for {}: {}", addr, e);
        }
    }
}
