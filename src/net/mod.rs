// Network connection scanning module
// Read-only operations following ntomb security-domain guidelines

use std::fs;
use std::io;

/// TCP connection states from /proc/net/tcp
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

impl ConnectionState {
    /// Parse hex state value from /proc/net/tcp
    fn from_hex(hex_str: &str) -> Self {
        match hex_str {
            "01" => ConnectionState::Established,
            "02" => ConnectionState::SynSent,
            "03" => ConnectionState::SynRecv,
            "04" => ConnectionState::FinWait1,
            "05" => ConnectionState::FinWait2,
            "06" => ConnectionState::TimeWait,
            "07" => ConnectionState::Close,
            "08" => ConnectionState::CloseWait,
            "09" => ConnectionState::LastAck,
            "0A" => ConnectionState::Listen,
            "0B" => ConnectionState::Closing,
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

impl Connection {
    /// Format connection for display
    pub fn format_display(&self) -> String {
        format!(
            "{}:{} -> {}:{} [{:?}]",
            self.local_addr, self.local_port, self.remote_addr, self.remote_port, self.state
        )
    }
}

/// Collect TCP connections from /proc/net/tcp
/// Read-only operation, never modifies system state
pub fn collect_connections() -> io::Result<Vec<Connection>> {
    let content = match fs::read_to_string("/proc/net/tcp") {
        Ok(c) => c,
        Err(e) => {
            // Gracefully handle permission or missing file errors
            // Following security-domain: calm, informative tone
            return Err(io::Error::new(
                e.kind(),
                format!("Cannot read /proc/net/tcp: {}", e),
            ));
        }
    };

    let mut connections = Vec::new();

    // Skip header line
    for line in content.lines().skip(1) {
        if let Some(conn) = parse_tcp_line(line) {
            connections.push(conn);
        }
    }

    Ok(connections)
}

/// Parse a single line from /proc/net/tcp
fn parse_tcp_line(line: &str) -> Option<Connection> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    // /proc/net/tcp format:
    // sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
    // 0: 0100007F:1F90 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 12345

    if parts.len() < 10 {
        return None;
    }

    // Parse local address (format: HEXIP:HEXPORT)
    let local = parse_address(parts[1])?;
    let remote = parse_address(parts[2])?;

    // Parse state
    let state = ConnectionState::from_hex(parts[3]);

    // Parse inode
    let inode = parts.get(9).and_then(|s| s.parse::<u64>().ok());

    Some(Connection {
        local_addr: local.0,
        local_port: local.1,
        remote_addr: remote.0,
        remote_port: remote.1,
        state,
        inode,
    })
}

/// Parse hex address:port format from /proc/net/tcp
/// Format: HEXIP:HEXPORT (e.g., "0100007F:1F90" = 127.0.0.1:8080)
fn parse_address(addr_str: &str) -> Option<(String, u16)> {
    let parts: Vec<&str> = addr_str.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let ip = parse_hex_ip(parts[0])?;
    let port = u16::from_str_radix(parts[1], 16).ok()?;

    Some((ip, port))
}

/// Parse hex IP address (little-endian format)
/// Example: "0100007F" = 127.0.0.1
fn parse_hex_ip(hex_ip: &str) -> Option<String> {
    if hex_ip.len() != 8 {
        return None;
    }

    let mut octets = Vec::new();
    for i in (0..8).step_by(2) {
        let octet = u8::from_str_radix(&hex_ip[i..i + 2], 16).ok()?;
        octets.push(octet);
    }

    // Reverse for little-endian
    octets.reverse();

    Some(format!(
        "{}.{}.{}.{}",
        octets[0], octets[1], octets[2], octets[3]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_ip() {
        assert_eq!(parse_hex_ip("0100007F"), Some("127.0.0.1".to_string()));
        assert_eq!(parse_hex_ip("00000000"), Some("0.0.0.0".to_string()));
    }

    #[test]
    fn test_parse_address() {
        let result = parse_address("0100007F:1F90");
        assert!(result.is_some());
        let (ip, port) = result.unwrap();
        assert_eq!(ip, "127.0.0.1");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_connection_state_from_hex() {
        assert_eq!(ConnectionState::from_hex("01"), ConnectionState::Established);
        assert_eq!(ConnectionState::from_hex("0A"), ConnectionState::Listen);
        assert_eq!(ConnectionState::from_hex("06"), ConnectionState::TimeWait);
    }
}
