//! TCP and UDP echo handlers for the Rucho server.
//!
//! This module provides handlers for raw TCP and UDP connections that echo
//! received data back to clients. These are useful for testing network connectivity
//! and debugging client applications.
//!
//! # Security Considerations
//!
//! - **Buffer limits**: All buffers are capped at `MAX_BUFFER_SIZE` to prevent
//!   memory exhaustion from malicious large payloads.
//! - **Exponential backoff**: UDP errors trigger exponential backoff to prevent
//!   hot loops that could consume excessive CPU resources.
//! - **Graceful error handling**: Connection errors are logged but don't crash
//!   the server.

use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use crate::utils::constants::{MAX_BUFFER_SIZE, UDP_ERROR_BACKOFF_BASE_MS, UDP_ERROR_BACKOFF_MAX_MS};

/// Handles an incoming TCP connection by echoing received data back to the client.
///
/// Reads data from the stream in chunks up to `MAX_BUFFER_SIZE`, logs it, and
/// writes it back. The loop continues until the client closes the connection
/// or an unrecoverable error occurs.
///
/// # Arguments
///
/// * `stream` - The TCP stream to handle
///
/// # Security
///
/// The read buffer is limited to `MAX_BUFFER_SIZE` (64KB) to prevent memory
/// exhaustion attacks from malicious clients sending large payloads.
pub async fn handle_tcp_connection(mut stream: TcpStream) {
    let peer_addr = match stream.peer_addr() {
        Ok(addr) => addr.to_string(),
        Err(_) => "unknown peer".to_string(),
    };
    tracing::info!("Accepted TCP connection from: {}", peer_addr);

    // Use a fixed-size buffer capped at MAX_BUFFER_SIZE for security
    let mut buf = vec![0u8; MAX_BUFFER_SIZE.min(65536)];

    loop {
        match stream.read(&mut buf).await {
            Ok(0) => {
                tracing::info!("TCP connection closed by client: {}", peer_addr);
                break;
            }
            Ok(n) => {
                tracing::info!(
                    "Received {} bytes from {}: {:?}",
                    n,
                    peer_addr,
                    String::from_utf8_lossy(&buf[..n])
                );

                if let Err(e) = stream.write_all(&buf[..n]).await {
                    tracing::error!("Failed to write to TCP stream for {}: {}", peer_addr, e);
                    break;
                }
                tracing::info!("Echoed {} bytes back to {}", n, peer_addr);
            }
            Err(e) => {
                tracing::error!("Failed to read from TCP stream for {}: {}", peer_addr, e);
                break;
            }
        }
    }
}

/// Handles UDP packets by echoing them back to the sender.
///
/// Continuously listens for packets on the provided UDP socket, logs them,
/// and sends them back to their origin. Implements exponential backoff on
/// consecutive errors to prevent hot loops.
///
/// # Arguments
///
/// * `socket` - Arc-wrapped UDP socket to listen on
///
/// # Returns
///
/// Returns `Ok(())` when the handler exits (which should be never under normal operation).
///
/// # Security
///
/// - Buffer size is limited to `MAX_BUFFER_SIZE` (64KB) to prevent memory issues.
/// - Exponential backoff is applied on consecutive errors to prevent CPU exhaustion
///   from error hot loops. Backoff starts at 100ms and caps at 5 seconds.
pub async fn handle_udp_socket(socket: Arc<UdpSocket>) -> std::io::Result<()> {
    let local_addr = match socket.local_addr() {
        Ok(addr) => addr.to_string(),
        Err(_) => "unknown local UDP socket".to_string(),
    };
    tracing::info!("UDP listener active on {}", local_addr);

    // Use a fixed-size buffer capped at MAX_BUFFER_SIZE for security
    let mut buf = vec![0u8; MAX_BUFFER_SIZE.min(65536)];
    let mut consecutive_errors: u32 = 0;

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((size, src_addr)) => {
                // Reset error counter on successful receive
                consecutive_errors = 0;

                tracing::info!(
                    "Received {} bytes from {} on UDP socket {}: {:?}",
                    size,
                    src_addr,
                    local_addr,
                    String::from_utf8_lossy(&buf[..size])
                );

                if let Err(e) = socket.send_to(&buf[..size], src_addr).await {
                    tracing::error!(
                        "Failed to send UDP packet to {} from {}: {}",
                        src_addr,
                        local_addr,
                        e
                    );
                } else {
                    tracing::info!(
                        "Echoed {} bytes back to {} via UDP from {}",
                        size,
                        src_addr,
                        local_addr
                    );
                }
            }
            Err(e) => {
                tracing::error!("Failed to receive UDP packet on {}: {}", local_addr, e);

                // Implement exponential backoff to prevent hot loop on persistent errors
                consecutive_errors = consecutive_errors.saturating_add(1);
                let backoff_ms = UDP_ERROR_BACKOFF_BASE_MS
                    .saturating_mul(2u64.saturating_pow(consecutive_errors.min(10)))
                    .min(UDP_ERROR_BACKOFF_MAX_MS);

                tracing::warn!(
                    "UDP error backoff: waiting {}ms before retry (consecutive errors: {})",
                    backoff_ms,
                    consecutive_errors
                );

                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }
        }
    }

    #[allow(unreachable_code)]
    Ok(())
}
