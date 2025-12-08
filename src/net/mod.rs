// Network connection scanning module
// Read-only operations following ntomb security-domain guidelines
// Uses netstat2 for cross-platform network socket information

use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};
use std::io;
use sysinfo::System;

#[cfg(target_os = "linux")]
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::net::{Ipv4Addr, Ipv6Addr};

/// TCP connection states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    #[allow(dead_code)]
    pub inode: Option<u64>,
    /// Process ID that owns this connection (populated by procfs on Linux)
    pub pid: Option<i32>,
    /// Process name that owns this connection (populated by procfs on Linux)
    pub process_name: Option<String>,
}

/// Collect TCP connections using netstat2
/// Cross-platform, read-only operation, never modifies system state
///
/// Uses netstat2's associated_pids for process information on all platforms,
/// and sysinfo to resolve PID to process name.
pub fn collect_connections() -> io::Result<Vec<Connection>> {
    // Query both IPv4 and IPv6 TCP connections
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP;

    let sockets = get_sockets_info(af_flags, proto_flags).map_err(|e| {
        // Gracefully handle errors
        // Following security-domain: calm, informative tone
        io::Error::other(format!("Cannot retrieve network sockets: {}", e))
    })?;

    // Initialize sysinfo for process name lookup
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let mut connections = Vec::new();

    for socket_info in sockets {
        if let ProtocolSocketInfo::Tcp(tcp_info) = socket_info.protocol_socket_info {
            // Get PID from netstat2's associated_pids (cross-platform!)
            let pid = socket_info.associated_pids.first().map(|&p| p as i32);

            // Lookup process name using sysinfo
            let process_name = pid.and_then(|p| {
                let sysinfo_pid = sysinfo::Pid::from_u32(p as u32);
                sys.process(sysinfo_pid)
                    .map(|proc| proc.name().to_string_lossy().to_string())
            });

            connections.push(Connection {
                local_addr: tcp_info.local_addr.to_string(),
                local_port: tcp_info.local_port,
                remote_addr: tcp_info.remote_addr.to_string(),
                remote_port: tcp_info.remote_port,
                state: ConnectionState::from(tcp_info.state),
                inode: None,
                pid,
                process_name,
            });
        }
    }

    // On Linux, populate inodes by reading /proc/net/tcp and /proc/net/tcp6
    #[cfg(target_os = "linux")]
    populate_inodes(&mut connections)?;

    Ok(connections)
}

/// On Linux, read /proc/net/tcp and /proc/net/tcp6 to get socket inodes
/// and match them to connections by local/remote address and port
#[cfg(target_os = "linux")]
fn populate_inodes(connections: &mut [Connection]) -> io::Result<()> {
    // Build a map of (local_addr, local_port, remote_addr, remote_port) -> inode
    let mut inode_map = HashMap::new();

    // Parse /proc/net/tcp (IPv4)
    if let Ok(content) = fs::read_to_string("/proc/net/tcp") {
        parse_proc_net_tcp(&content, &mut inode_map, false);
    }

    // Parse /proc/net/tcp6 (IPv6)
    if let Ok(content) = fs::read_to_string("/proc/net/tcp6") {
        parse_proc_net_tcp(&content, &mut inode_map, true);
    }

    // Match connections to inodes
    for conn in connections.iter_mut() {
        let key = (
            conn.local_addr.clone(),
            conn.local_port,
            conn.remote_addr.clone(),
            conn.remote_port,
        );
        if let Some(&inode) = inode_map.get(&key) {
            conn.inode = Some(inode);
        }
    }

    Ok(())
}

/// Parse /proc/net/tcp or /proc/net/tcp6 format
/// Format: sl local_address rem_address st tx_queue rx_queue tr tm->when retrnsmt uid timeout inode
#[cfg(target_os = "linux")]
fn parse_proc_net_tcp(
    content: &str,
    inode_map: &mut HashMap<(String, u16, String, u16), u64>,
    is_ipv6: bool,
) {
    for line in content.lines().skip(1) {
        // Skip header line
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }

        // Parse local address (format: "0100007F:1F90" for 127.0.0.1:8080)
        let local_parts: Vec<&str> = parts[1].split(':').collect();
        if local_parts.len() != 2 {
            continue;
        }
        let local_addr = parse_hex_addr(local_parts[0], is_ipv6);
        let local_port = u16::from_str_radix(local_parts[1], 16).unwrap_or(0);

        // Parse remote address
        let remote_parts: Vec<&str> = parts[2].split(':').collect();
        if remote_parts.len() != 2 {
            continue;
        }
        let remote_addr = parse_hex_addr(remote_parts[0], is_ipv6);
        let remote_port = u16::from_str_radix(remote_parts[1], 16).unwrap_or(0);

        // Parse inode (last field)
        if let Ok(inode) = parts[9].parse::<u64>() {
            inode_map.insert((local_addr, local_port, remote_addr, remote_port), inode);
        }
    }
}

/// Parse hex-encoded IP address from /proc/net/tcp format
/// IPv4: "0100007F" = 127.0.0.1 (little-endian)
/// IPv6: "00000000000000000000000001000000" = ::1 (little-endian)
#[cfg(target_os = "linux")]
fn parse_hex_addr(hex: &str, is_ipv6: bool) -> String {
    if is_ipv6 {
        // IPv6: 32 hex chars = 16 bytes
        if hex.len() != 32 {
            return "::".to_string();
        }

        // Parse as 4 u32 values in little-endian
        let mut bytes = [0u8; 16];
        for i in 0..4 {
            let start = i * 8;
            let end = start + 8;
            if let Ok(val) = u32::from_str_radix(&hex[start..end], 16) {
                let val_bytes = val.to_le_bytes();
                bytes[i * 4] = val_bytes[0];
                bytes[i * 4 + 1] = val_bytes[1];
                bytes[i * 4 + 2] = val_bytes[2];
                bytes[i * 4 + 3] = val_bytes[3];
            }
        }

        let addr = Ipv6Addr::from(bytes);
        addr.to_string()
    } else {
        // IPv4: 8 hex chars = 4 bytes in little-endian
        if hex.len() != 8 {
            return "0.0.0.0".to_string();
        }

        if let Ok(val) = u32::from_str_radix(hex, 16) {
            let bytes = val.to_le_bytes();
            let addr = Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]);
            addr.to_string()
        } else {
            "0.0.0.0".to_string()
        }
    }
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
                assert!(!conns.is_empty());
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

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_hex_addr_ipv4() {
        // Test localhost (127.0.0.1) in little-endian hex
        assert_eq!(parse_hex_addr("0100007F", false), "127.0.0.1");

        // Test 0.0.0.0
        assert_eq!(parse_hex_addr("00000000", false), "0.0.0.0");

        // Test 192.168.1.1 (0xC0A80101 in big-endian = 0x0101A8C0 in little-endian)
        assert_eq!(parse_hex_addr("0101A8C0", false), "192.168.1.1");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_hex_addr_ipv6() {
        // Test localhost (::1)
        assert_eq!(
            parse_hex_addr("00000000000000000000000001000000", true),
            "::1"
        );

        // Test :: (all zeros)
        assert_eq!(
            parse_hex_addr("00000000000000000000000000000000", true),
            "::"
        );
    }
}
