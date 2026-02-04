//! UDP echo server setup.

use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::task::JoinHandle;

use crate::tcp_udp_handlers::handle_udp_socket;

/// Binds a UDP socket to the given address.
///
/// # Arguments
///
/// * `udp_addr_str` - The address string to bind to (e.g., "127.0.0.1:9000")
///
/// # Returns
///
/// `Some(UdpSocket)` if binding succeeds, `None` otherwise.
pub async fn bind_udp_socket(udp_addr_str: &str) -> Option<UdpSocket> {
    let addr: std::net::SocketAddr = match udp_addr_str.parse() {
        Ok(addr) => addr,
        Err(e) => {
            tracing::error!("Failed to parse UDP address '{}': {}", udp_addr_str, e);
            return None;
        }
    };

    match UdpSocket::bind(addr).await {
        Ok(socket) => {
            tracing::info!("Bound UDP socket on {}", addr);
            Some(socket)
        }
        Err(e) => {
            tracing::error!("Failed to bind UDP listener for {}: {}", addr, e);
            None
        }
    }
}

/// Sets up a UDP echo listener using the given socket.
///
/// Spawns a task that receives UDP packets and echoes them back to the sender.
pub fn setup_udp_listener(
    socket: Arc<UdpSocket>,
    server_handles: &mut Vec<JoinHandle<Result<(), std::io::Error>>>,
) {
    let local_addr = socket
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    tracing::info!("Starting UDP echo listener on {}", local_addr);
    let udp_handle = tokio::spawn(handle_udp_socket(socket));
    server_handles.push(udp_handle);
}
