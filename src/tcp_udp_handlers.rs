use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tracing;

/// Handles an incoming TCP connection by echoing received data back to the client.
///
/// It reads data from the stream, logs it, and writes it back. The loop continues
/// until the client closes the connection or an error occurs.
pub async fn handle_tcp_connection(mut stream: TcpStream) {
    let peer_addr = match stream.peer_addr() {
        Ok(addr) => addr.to_string(),
        Err(_) => "unknown peer".to_string(),
    };
    tracing::info!("Accepted TCP connection from: {}", peer_addr);

    let mut buf = Vec::with_capacity(1024); // Using Vec<u8> for read_buf

    loop {
        buf.clear(); // Clear buffer for new read
        match stream.read_buf(&mut buf).await {
            Ok(0) => {
                tracing::info!("TCP connection closed by client: {}", peer_addr);
                break;
            }
            Ok(n) => {
                // Data is already in buf, no need to slice if using read_buf correctly
                tracing::info!("Received {} bytes from {}: {:?}", n, peer_addr, String::from_utf8_lossy(&buf));

                if let Err(e) = stream.write_all(&buf).await {
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
/// It continuously listens for packets on the provided UDP socket, logs them,
/// and sends them back to their origin. The function is designed to run indefinitely.
pub async fn handle_udp_socket(socket: Arc<UdpSocket>) -> std::io::Result<()> {
    let local_addr = match socket.local_addr() {
        Ok(addr) => addr.to_string(),
        Err(_) => "unknown local UDP socket".to_string(),
    };
    tracing::info!("UDP listener active on {}", local_addr);

    let mut buf = [0; 1024];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((size, src_addr)) => {
                tracing::info!("Received {} bytes from {} on UDP socket {}: {:?}", size, src_addr, local_addr, String::from_utf8_lossy(&buf[..size]));

                if let Err(e) = socket.send_to(&buf[..size], src_addr).await {
                    tracing::error!("Failed to send UDP packet to {} from {}: {}", src_addr, local_addr, e);
                    // Log error and continue loop
                } else {
                    tracing::info!("Echoed {} bytes back to {} via UDP from {}", size, src_addr, local_addr);
                }
            }
            Err(e) => {
                tracing::error!("Failed to receive UDP packet on {}: {}", local_addr, e);
                // Log error and continue loop.
                // Some errors might be critical, but for an echo server, we'll try to continue.
                // If recv_from itself fails consistently, the loop might become a hot loop.
                // A more robust solution might implement a backoff strategy for certain errors.
            }
        }
    }
    // This part is unreachable for a service that loops indefinitely but is needed for type compatibility.
    #[allow(unreachable_code)]
    Ok(())
}
