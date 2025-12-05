// Kiroween Overdrive theme functions
//
// This module provides the Halloween-themed overdrive mode functions for ntomb.
// When Kiroween Overdrive mode is enabled, these functions return enhanced
// Halloween-themed icons and text that add personality while maintaining clarity.
//
// Requirements: 2.3, 4.2, 4.3, 4.4, 4.5

use crate::app::LatencyBucket;
use crate::net::ConnectionState;

use super::get_normal_status_text;

/// Get overdrive-themed icon based on connection state and latency
///
/// When Kiroween Overdrive mode is enabled, this function returns enhanced
/// Halloween-themed icons that add personality while maintaining clarity.
///
/// Icon mappings:
/// - ESTABLISHED (healthy) → "🟢👻" (ghost haunting the connection)
/// - High latency → "🔥🎃" (fire-pumpkin indicating heat/slowness)
/// - CLOSE_WAIT/TIME_WAIT → "💀" (skull for dying connections)
/// - Other states → standard icons
///
/// # Arguments
/// * `state` - The connection state
/// * `latency_bucket` - The latency classification for the connection
///
/// # Returns
/// A static string containing the overdrive-themed icon
///
/// Requirements: 4.2, 4.3, 4.4
pub fn get_overdrive_icon(state: ConnectionState, latency_bucket: LatencyBucket) -> &'static str {
    // Priority: CLOSE_WAIT/TIME_WAIT > High latency > ESTABLISHED > Other
    match state {
        // Dying connections get skull icon (Requirement 4.4)
        ConnectionState::CloseWait | ConnectionState::TimeWait => "💀",

        // Established connections: check latency first
        ConnectionState::Established => {
            // High latency gets fire-pumpkin (Requirement 4.3)
            if latency_bucket == LatencyBucket::High {
                "🔥🎃"
            } else {
                // Healthy established connections get ghost (Requirement 4.2)
                "🟢👻"
            }
        }

        // Other states with high latency also get fire-pumpkin
        _ => {
            if latency_bucket == LatencyBucket::High {
                "🔥🎃"
            } else {
                // Default to standard state indicator
                match state {
                    ConnectionState::Listen => "🕯",
                    ConnectionState::SynSent | ConnectionState::SynRecv => "⏳",
                    ConnectionState::Close => "💀",
                    ConnectionState::FinWait1 | ConnectionState::FinWait2 => "👻",
                    ConnectionState::LastAck | ConnectionState::Closing => "👻",
                    _ => "❓",
                }
            }
        }
    }
}

/// Get overdrive-themed status text for connection states
///
/// When Kiroween Overdrive mode is enabled, this function returns themed
/// status descriptions that maintain a calm, informative tone while adding
/// Halloween personality.
///
/// Text mappings:
/// - "Alive" → "Haunting" (active connections are haunting the network)
/// - "Listening" → "Summoning" (server sockets summoning connections)
/// - "Closing" → "Fading" (dying connections fading away)
///
/// # Arguments
/// * `state` - The connection state
///
/// # Returns
/// A static string containing the overdrive-themed status text
///
/// Requirements: 4.5
pub fn get_overdrive_status_text(state: ConnectionState) -> &'static str {
    match state {
        // Active/healthy connections are "haunting" the network
        ConnectionState::Established => "Haunting",

        // Server sockets are "summoning" new connections
        ConnectionState::Listen => "Summoning",

        // Closing states are "fading" away
        ConnectionState::TimeWait
        | ConnectionState::CloseWait
        | ConnectionState::FinWait1
        | ConnectionState::FinWait2
        | ConnectionState::LastAck
        | ConnectionState::Closing
        | ConnectionState::Close => "Fading",

        // Connection attempts are "awakening"
        ConnectionState::SynSent | ConnectionState::SynRecv => "Awakening",

        // Unknown states
        ConnectionState::Unknown => "Unknown",
    }
}

/// Get the appropriate stats label based on overdrive mode
///
/// When Kiroween Overdrive mode is enabled, connection counts are referred
/// to as "Spirits" instead of "Connections" to enhance the Halloween theme.
///
/// # Arguments
/// * `overdrive_enabled` - Whether Kiroween Overdrive mode is active
///
/// # Returns
/// "Spirits" if overdrive is enabled, "Connections" otherwise
///
/// Requirements: 4.5
pub fn get_stats_label(overdrive_enabled: bool) -> &'static str {
    if overdrive_enabled {
        "Spirits"
    } else {
        "Connections"
    }
}

/// Get status text based on overdrive mode setting
///
/// Convenience function that returns either overdrive or normal status text
/// based on the current mode setting.
///
/// # Arguments
/// * `state` - The connection state
/// * `overdrive_enabled` - Whether Kiroween Overdrive mode is active
///
/// # Returns
/// Themed status text if overdrive is enabled, standard text otherwise
///
/// Requirements: 4.5
pub fn get_status_text(state: ConnectionState, overdrive_enabled: bool) -> &'static str {
    if overdrive_enabled {
        get_overdrive_status_text(state)
    } else {
        get_normal_status_text(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_overdrive_icon() {
        // Test ESTABLISHED with normal latency (Requirement 4.2)
        assert_eq!(
            get_overdrive_icon(ConnectionState::Established, LatencyBucket::Low),
            "🟢👻"
        );
        assert_eq!(
            get_overdrive_icon(ConnectionState::Established, LatencyBucket::Medium),
            "🟢👻"
        );

        // Test high latency (Requirement 4.3)
        assert_eq!(
            get_overdrive_icon(ConnectionState::Established, LatencyBucket::High),
            "🔥🎃"
        );

        // Test CLOSE_WAIT/TIME_WAIT (Requirement 4.4)
        assert_eq!(
            get_overdrive_icon(ConnectionState::CloseWait, LatencyBucket::Low),
            "💀"
        );
        assert_eq!(
            get_overdrive_icon(ConnectionState::TimeWait, LatencyBucket::Medium),
            "💀"
        );
    }

    #[test]
    fn test_get_overdrive_status_text() {
        // Test status text transformations (Requirement 4.5)
        assert_eq!(
            get_overdrive_status_text(ConnectionState::Established),
            "Haunting"
        );
        assert_eq!(
            get_overdrive_status_text(ConnectionState::Listen),
            "Summoning"
        );
        assert_eq!(
            get_overdrive_status_text(ConnectionState::TimeWait),
            "Fading"
        );
        assert_eq!(
            get_overdrive_status_text(ConnectionState::CloseWait),
            "Fading"
        );
    }

    #[test]
    fn test_get_stats_label() {
        // Test stats label based on overdrive mode (Requirement 4.5)
        assert_eq!(get_stats_label(false), "Connections");
        assert_eq!(get_stats_label(true), "Spirits");
    }

    #[test]
    fn test_get_status_text() {
        // Test normal mode
        assert_eq!(
            get_status_text(ConnectionState::Established, false),
            "Alive"
        );
        assert_eq!(get_status_text(ConnectionState::Listen, false), "Listening");

        // Test overdrive mode
        assert_eq!(
            get_status_text(ConnectionState::Established, true),
            "Haunting"
        );
        assert_eq!(get_status_text(ConnectionState::Listen, true), "Summoning");
    }
}
