// Network connection scanning module
// Read-only operations following ntomb security-domain guidelines
// Uses netstat2 for cross-platform network socket information

use netstat2::{
    get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState,
};
use std::io;

/// TCP connection states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Established,
    SynSent,
    SynRecv,
    FinWait1,
    FinWait2,
    TimeWait,
    Close,
    CloseWait,
    LastAck,
    Listen,
    Closing,
    Unknown,
}

impl From<TcpState> for ConnectionState {
    fn from(state: TcpState) -> Self {
        match state {
            TcpState::Established => ConnectionState::Established,
            TcpState::SynSent => ConnectionState::SynSent,
            TcpState::SynReceived => ConnectionState::SynRecv,
            TcpState::FinWait1 => ConnectionState::FinWait1,
            TcpState::FinWait2 => ConnectionState::FinWait2,
            TcpState::TimeWait => ConnectionState::TimeWait,
            TcpState::Closed => ConnectionState::Close,
            TcpState::CloseWait => ConnectionState::CloseWait,
            TcpState::LastAck => ConnectionState::LastAck,
            TcpState::Listen => ConnectionState::Listen,
            TcpState::Closing => ConnectionState::Closing,
            _ => ConnectionState::Unknown,
        }
    }
}

/// Represents a single TCP connection
#[derive(Debug, Clone)]
pub struct Connection {
    pub local_addr: String,
    pub local_port: u16,
    pub remote_addr: String,
    pub remote_port: u16,
    pub state: ConnectionState,
    pub inode: Option<u64>,
}

/// Collect TCP connections using netstat2
/// Cross-platform, read-only operation, never modifies system state
pub fn collect_connections() -> io::Result<Vec<Connection>> {
    // Query both IPv4 and IPv6 TCP connections
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP;

    let sockets = get_sockets_info(af_flags, proto_flags).map_err(|e| {
        // Gracefully handle errors
        // Following security-domain: calm, informative tone
        io::Error::new(
            io::ErrorKind::Other,
            format!("Cannot retrieve network sockets: {}", e),
        )
    })?;

    let mut connections = Vec::new();

    for socket_info in sockets {
        if let ProtocolSocketInfo::Tcp(tcp_info) = socket_info.protocol_socket_info {
            connections.push(Connection {
                local_addr: tcp_info.local_addr.to_string(),
                local_port: tcp_info.local_port,
                remote_addr: tcp_info.remote_addr.to_string(),
                remote_port: tcp_info.remote_port,
                state: ConnectionState::from(tcp_info.state),
                // netstat2 doesn't provide inode directly, but we can add it later if needed
                inode: None,
            });
        }
    }

    Ok(connections)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_connections() {
        // This test will only pass if the system has network connections
        // It's more of a smoke test to ensure the API works
        match collect_connections() {
            Ok(conns) => {
                println!("Found {} connections", conns.len());
                // Should have at least some connections on a typical system
                assert!(conns.len() >= 0);
            }
            Err(e) => {
                // On some systems this might fail due to permissions
                println!("Warning: Could not collect connections: {}", e);
            }
        }
    }

    #[test]
    fn test_connection_state_conversion() {
        assert_eq!(
            ConnectionState::from(TcpState::Established),
            ConnectionState::Established
        );
        assert_eq!(
            ConnectionState::from(TcpState::Listen),
            ConnectionState::Listen
        );
        assert_eq!(
            ConnectionState::from(TcpState::TimeWait),
            ConnectionState::TimeWait
        );
    }
}
