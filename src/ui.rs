// UI rendering module

use crate::app::{AppState, LatencyBucket, LatencyConfig};
use crate::net::ConnectionState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Line as CanvasLine},
        Block, Borders, BorderType, List, ListItem, Paragraph, Sparkline,
    },
    Frame,
};
use std::collections::HashMap;

// Color constants from ntomb-visual-design.md
const NEON_PURPLE: Color = Color::Rgb(187, 154, 247);
const PUMPKIN_ORANGE: Color = Color::Rgb(255, 158, 100);
const BLOOD_RED: Color = Color::Rgb(247, 118, 142);
const TOXIC_GREEN: Color = Color::Rgb(158, 206, 106);
const BONE_WHITE: Color = Color::Rgb(169, 177, 214);

// Latency ring constants for Graveyard visualization (Requirements 1.1, 1.6)
// Ring radii in virtual canvas space (0-100)
// Inner ring (Low latency < 50ms), Middle ring (Medium 50-200ms), Outer ring (High > 200ms)
const RING_RADII: [f64; 3] = [15.0, 25.0, 35.0];

// Center point of the HOST node in virtual canvas space
const HOST_CENTER: (f64, f64) = (50.0, 50.0);

// Edge particle animation constants (Requirements 2.1, 2.2)
// Offset positions for particles along the edge (0.0 to 1.0)
// 3 particles evenly distributed: start, 1/3, 2/3 along the edge
const PARTICLE_OFFSETS: [f32; 3] = [0.0, 0.33, 0.66];

// Symbol used to render particles on edges
const PARTICLE_SYMBOL: &str = "●";

// Performance optimization constants (Requirements 6.3, 6.4, 6.5)
// Maximum number of endpoints to display in the Graveyard canvas
// Prevents canvas overflow and maintains performance with many connections
const MAX_VISIBLE_ENDPOINTS: usize = 30;

// Threshold for reducing particle count to maintain performance
// When edge count exceeds this, reduce particles per edge
const PARTICLE_REDUCTION_THRESHOLD: usize = 50;

// Reduced particle offsets for high edge count scenarios
// Uses 1 particle instead of 3 to reduce rendering load
const REDUCED_PARTICLE_OFFSETS: [f32; 1] = [0.33];

/// Classification of endpoint types for visual rendering
/// 
/// Determines the icon and color used to display endpoints in the Graveyard
/// based on their IP address characteristics.
/// 
/// Requirements: 3.1, 3.2, 3.3, 3.5
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndpointType {
    /// Local loopback connections (127.0.0.1, ::1, 0.0.0.0)
    /// Icon: ⚰️, Color: Toxic Green
    Localhost,
    
    /// RFC1918 private IP addresses (10.x, 172.16-31.x, 192.168.x)
    /// Icon: 🪦, Color: Bone White
    Private,
    
    /// Public/external IP addresses (all non-private, non-localhost)
    /// Icon: 🎃, Color: Pumpkin Orange
    Public,
    
    /// Local server sockets in LISTEN state (no remote connection)
    /// Icon: 🕯, Color: Neon Purple
    ListenOnly,
}

impl EndpointType {
    /// Get the icon for this endpoint type
    /// 
    /// Returns the appropriate Halloween-themed emoji icon based on endpoint classification.
    /// 
    /// Requirements: 3.1, 3.2, 3.3, 3.5
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Localhost => "⚰️",
            Self::Private => "🪦",
            Self::Public => "🎃",
            Self::ListenOnly => "🕯",
        }
    }
    
    /// Get the primary color for this endpoint type
    /// 
    /// Returns the color from the approved palette for visual consistency.
    /// 
    /// Requirements: 3.1, 3.2, 3.3, 3.5
    pub fn color(&self) -> Color {
        match self {
            Self::Localhost => TOXIC_GREEN,
            Self::Private => BONE_WHITE,
            Self::Public => PUMPKIN_ORANGE,
            Self::ListenOnly => NEON_PURPLE,
        }
    }
    
    /// Get the icon with optional heavy talker badge
    /// 
    /// Returns the endpoint type icon with "👑" appended if the endpoint
    /// is a heavy talker (top 5 by connection count).
    /// 
    /// # Arguments
    /// * `is_heavy_talker` - Whether this endpoint is in the top 5 by connection count
    /// 
    /// # Returns
    /// A String containing the icon, with "👑" badge appended for heavy talkers
    /// 
    /// Requirements: 3.4
    pub fn icon_with_badge(&self, is_heavy_talker: bool) -> String {
        let base_icon = self.icon();
        if is_heavy_talker {
            format!("{}👑", base_icon)
        } else {
            base_icon.to_string()
        }
    }
}

/// Classify an endpoint IP address into an EndpointType
/// 
/// Classification logic:
/// 1. Localhost: 127.0.0.1, ::1, or 0.0.0.0
/// 2. Private: RFC1918 ranges (10.x, 172.16-31.x, 192.168.x)
/// 3. ListenOnly: When remote address is 0.0.0.0:0 (LISTEN socket)
/// 4. Public: All other IP addresses
/// 
/// # Arguments
/// * `ip` - The IP address string to classify
/// * `is_listen_socket` - True if this is a LISTEN-only socket (remote = 0.0.0.0:0)
/// 
/// # Returns
/// The appropriate EndpointType classification
/// 
/// Requirements: 3.1, 3.2, 3.3, 3.5
pub fn classify_endpoint(ip: &str, is_listen_socket: bool) -> EndpointType {
    // Check for LISTEN-only sockets first (remote = 0.0.0.0:0)
    // These are local server sockets waiting for connections
    if is_listen_socket {
        return EndpointType::ListenOnly;
    }
    
    // Check for localhost addresses
    // Includes IPv4 loopback (127.0.0.1), IPv6 loopback (::1), and wildcard (0.0.0.0)
    if ip == "127.0.0.1" || ip == "::1" || ip == "0.0.0.0" {
        return EndpointType::Localhost;
    }
    
    // Check for RFC1918 private IP ranges
    // Parse as IPv4 and check against private ranges
    if let Some(endpoint_type) = classify_ipv4_private(ip) {
        return endpoint_type;
    }
    
    // Default to Public for all other addresses
    EndpointType::Public
}

/// Helper function to classify IPv4 addresses against RFC1918 private ranges
/// 
/// RFC1918 private ranges:
/// - 10.0.0.0/8 (10.0.0.0 - 10.255.255.255)
/// - 172.16.0.0/12 (172.16.0.0 - 172.31.255.255)
/// - 192.168.0.0/16 (192.168.0.0 - 192.168.255.255)
/// 
/// Returns Some(EndpointType::Private) if the IP is in a private range,
/// None otherwise.
fn classify_ipv4_private(ip: &str) -> Option<EndpointType> {
    // Parse the IP address into octets
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return None; // Not a valid IPv4 address
    }
    
    // Parse each octet
    let octets: Vec<u8> = parts
        .iter()
        .filter_map(|p| p.parse::<u8>().ok())
        .collect();
    
    if octets.len() != 4 {
        return None; // Failed to parse all octets
    }
    
    // Check RFC1918 private ranges
    
    // 10.0.0.0/8 - Class A private network
    if octets[0] == 10 {
        return Some(EndpointType::Private);
    }
    
    // 172.16.0.0/12 - Class B private network (172.16.x.x - 172.31.x.x)
    if octets[0] == 172 && (16..=31).contains(&octets[1]) {
        return Some(EndpointType::Private);
    }
    
    // 192.168.0.0/16 - Class C private network
    if octets[0] == 192 && octets[1] == 168 {
        return Some(EndpointType::Private);
    }
    
    None
}

/// Determine if an endpoint is a "heavy talker" based on connection count
/// 
/// An endpoint is considered a heavy talker if its connection count is in the
/// top 5 among all endpoints. This helps identify endpoints with unusually
/// high activity that may warrant investigation.
/// 
/// # Arguments
/// * `conn_count` - The connection count for the endpoint being checked
/// * `all_counts` - A slice of all endpoint connection counts for comparison
/// 
/// # Returns
/// `true` if the endpoint is in the top 5 by connection count, `false` otherwise
/// 
/// # Edge Cases
/// - If there are fewer than 5 endpoints, all endpoints are considered heavy talkers
/// - If multiple endpoints have the same count as the 5th highest, all are included
/// 
/// Requirements: 3.4
pub fn is_heavy_talker(conn_count: usize, all_counts: &[usize]) -> bool {
    if all_counts.is_empty() {
        return false;
    }
    
    // Sort counts in descending order to find top 5
    let mut sorted = all_counts.to_vec();
    sorted.sort_by(|a, b| b.cmp(a));
    
    // Determine the threshold for top 5
    // If fewer than 5 endpoints, use the minimum count (all are heavy talkers)
    let threshold = if sorted.len() >= 5 {
        sorted[4] // 5th highest count (0-indexed)
    } else {
        // Fewer than 5 endpoints - use the lowest count
        *sorted.last().unwrap_or(&0)
    };
    
    // An endpoint is a heavy talker if its count >= threshold
    conn_count >= threshold && conn_count > 0
}

/// Classify latency into buckets for ring positioning
/// 
/// Maps latency values to LatencyBucket based on configured thresholds:
/// - Low: < low_threshold_ms (default 50ms) - innermost ring
/// - Medium: low_threshold_ms to high_threshold_ms (default 50-200ms) - middle ring
/// - High: > high_threshold_ms (default 200ms) - outermost ring
/// - Unknown: No latency data available - use default position
/// 
/// Requirements: 1.2, 1.3, 1.4, 1.5
pub fn classify_latency(latency_ms: Option<u64>, config: &LatencyConfig) -> LatencyBucket {
    match latency_ms {
        None => LatencyBucket::Unknown,
        Some(ms) => {
            if ms < config.low_threshold_ms {
                LatencyBucket::Low
            } else if ms <= config.high_threshold_ms {
                LatencyBucket::Medium
            } else {
                LatencyBucket::High
            }
        }
    }
}

/// Calculate particle position along an edge for spirit flow animation
/// 
/// Uses linear interpolation to position a particle along the line segment
/// from `start` to `end`. The position is determined by combining the
/// `pulse_phase` (animation time 0.0-1.0) with a particle `offset` to
/// create multiple evenly-spaced particles moving along the edge.
/// 
/// # Arguments
/// * `start` - Starting point (x, y) of the edge (typically HOST_CENTER)
/// * `end` - Ending point (x, y) of the edge (endpoint position)
/// * `pulse_phase` - Current animation phase (0.0 to 1.0, cycles over time)
/// * `offset` - Particle offset along the edge (0.0, 0.33, 0.66 for 3 particles)
/// 
/// # Returns
/// (x, y) coordinates of the particle position in canvas space
/// 
/// # Example
/// ```ignore
/// // Get position of first particle (offset 0.0) at animation phase 0.5
/// let pos = particle_position((50.0, 50.0), (80.0, 30.0), 0.5, 0.0);
/// // Returns (65.0, 40.0) - midpoint of the edge
/// ```
/// 
/// Requirements: 2.2
pub fn particle_position(
    start: (f64, f64),
    end: (f64, f64),
    pulse_phase: f32,
    offset: f32,
) -> (f64, f64) {
    // Calculate parametric position t along the edge (0.0 to 1.0)
    // Wrapping with modulo ensures smooth cycling animation
    let t = ((pulse_phase + offset) % 1.0) as f64;
    
    // Linear interpolation between start and end points
    let x = start.0 + (end.0 - start.0) * t;
    let y = start.1 + (end.1 - start.1) * t;
    
    (x, y)
}

/// Minimum canvas height (in canvas units) to use the large coffin design
/// Below this threshold, the mini coffin (single line) is used
const LARGE_COFFIN_MIN_HEIGHT: f64 = 50.0;

/// Draw the coffin block on the canvas at the HOST center
/// 
/// Renders a hexagonal coffin shape for the central HOST node.
/// Automatically switches to mini (single-line) mode when canvas is small.
/// 
/// Large coffin (4 lines):
/// ```text
///    ╱‾‾‾‾‾‾‾‾‾‾‾‾╲
///   ╱  ⚰ HOST_NAME ╲
///   ╲              ╱
///    ╲____________╱
/// ```
/// 
/// Mini coffin (1 line, for small screens):
/// ```text
/// ⚰ HOST_NAME
/// ```
/// 
/// # Arguments
/// * `ctx` - The canvas context for drawing
/// * `host_name` - The name to display (e.g., "HOST", "kafka-broker-1")
/// * `overdrive_enabled` - When true, uses Pumpkin Orange for a "burning" effect
/// * `canvas_height` - Height of the canvas in canvas units (used to determine coffin size)
pub fn draw_coffin_block(
    ctx: &mut ratatui::widgets::canvas::Context<'_>,
    host_name: &str,
    overdrive_enabled: bool,
    canvas_height: f64,
) {
    let (cx, cy) = HOST_CENTER;
    
    // Coffin color: Neon Purple normally, Pumpkin Orange in overdrive mode
    // Using Neon Purple instead of Bone White for better visibility on both
    // light and dark terminal backgrounds
    let coffin_color = if overdrive_enabled {
        PUMPKIN_ORANGE
    } else {
        NEON_PURPLE
    };
    
    // Truncate host name if too long (max 10 chars for display)
    let display_name = if host_name.len() > 10 {
        format!("{}...", &host_name[..7])
    } else {
        host_name.to_string()
    };
    
    // Choose coffin size based on canvas height
    if canvas_height >= LARGE_COFFIN_MIN_HEIGHT {
        draw_large_coffin(ctx, cx, cy, &display_name, coffin_color);
    } else {
        draw_mini_coffin(ctx, cx, cy, &display_name, coffin_color);
    }
}

/// Draw the large hexagonal coffin (4 lines)
/// Uses widely-supported box-drawing characters for better terminal compatibility
/// ```text
///   /‾‾‾‾‾‾‾‾‾‾‾‾‾‾\
///  /   ⚰ HOST_NAME  \
///  \                /
///   \______________/
/// ```
fn draw_large_coffin(
    ctx: &mut ratatui::widgets::canvas::Context<'_>,
    cx: f64,
    cy: f64,
    display_name: &str,
    coffin_color: Color,
) {
    // Calculate widths based on name length
    // Inner content: "   ⚰ " + name + "  " = 6 + name_len
    let content_width = 6 + display_name.len();
    let top_bar_width = content_width; // Same width for consistency
    let bottom_bar_width = content_width;
    
    // Build coffin lines using ASCII-compatible characters
    // Using / \ instead of ╱ ╲ for better terminal compatibility
    // Using ‾ (macron/overline) for top bar - fallback to - if needed
    let line1 = format!(" /{}\\", "─".repeat(top_bar_width));
    let line2 = format!("/   ⚰ {}  \\", display_name);
    let line3 = format!("\\{}/ ", " ".repeat(content_width));
    let line4 = format!(" \\{}/", "─".repeat(bottom_bar_width));
    
    // Calculate centering
    let cell_width = 0.8;
    let max_line_width = line2.chars().count() as f64 * cell_width;
    let base_x = cx - max_line_width / 2.0;
    
    // Vertical spacing between lines
    let line_spacing = 3.5;
    let start_y = cy + line_spacing * 1.5;
    
    let style = Style::default().fg(coffin_color).add_modifier(Modifier::BOLD);
    
    // Draw all 4 lines
    ctx.print(base_x + cell_width, start_y, Span::styled(line1, style));
    ctx.print(base_x, start_y - line_spacing, Span::styled(line2, style));
    ctx.print(base_x, start_y - line_spacing * 2.0, Span::styled(line3, style));
    ctx.print(base_x + cell_width, start_y - line_spacing * 3.0, Span::styled(line4, style));
}

/// Draw the mini coffin (single line for small screens)
/// ```text
/// ⚰ HOST_NAME
/// ```
fn draw_mini_coffin(
    ctx: &mut ratatui::widgets::canvas::Context<'_>,
    cx: f64,
    cy: f64,
    display_name: &str,
    coffin_color: Color,
) {
    let content = format!("⚰ {}", display_name);
    
    // Calculate centering
    // ⚰ emoji is 2 cells wide, rest are 1 cell each
    let display_width = 2 + 1 + display_name.len(); // emoji + space + name
    let cell_width = 0.8;
    let content_width = display_width as f64 * cell_width;
    let x = cx - content_width / 2.0;
    
    ctx.print(
        x,
        cy,
        Span::styled(
            content,
            Style::default().fg(coffin_color).add_modifier(Modifier::BOLD),
        ),
    );
}

/// Draw latency rings on the canvas around the HOST center
/// 
/// Draws 3 concentric dotted circles using Braille markers:
/// - Inner ring (radius 15): Low latency endpoints (< 50ms)
/// - Middle ring (radius 25): Medium latency endpoints (50-200ms)
/// - Outer ring (radius 35): High latency endpoints (> 200ms)
/// 
/// Uses Bone White color with decreasing opacity for outer rings
/// to avoid visual clutter while maintaining visibility.
/// 
/// Requirements: 1.1, 1.6
pub fn draw_latency_rings<F>(ctx: &mut ratatui::widgets::canvas::Context<'_>, draw_point: F)
where
    F: Fn(&mut ratatui::widgets::canvas::Context<'_>, f64, f64, Style),
{
    let (cx, cy) = HOST_CENTER;
    
    for (ring_idx, radius) in RING_RADII.iter().enumerate() {
        // Calculate opacity: inner ring is brightest, outer rings fade
        // Base Bone White RGB: (169, 177, 214)
        // Decrease brightness for outer rings to avoid visual clutter
        let opacity_factor = 1.0 - (ring_idx as f32 * 0.25); // 1.0, 0.75, 0.5
        let r = (169.0 * opacity_factor) as u8;
        let g = (177.0 * opacity_factor) as u8;
        let b = (214.0 * opacity_factor) as u8;
        let ring_color = Color::Rgb(r, g, b);
        let ring_style = Style::default().fg(ring_color);
        
        // Draw ring as series of dotted points (every 10 degrees for dotted effect)
        // Using step of 10 degrees creates a dotted/dashed appearance
        for angle_deg in (0..360).step_by(10) {
            let angle_rad = (angle_deg as f64).to_radians();
            let x = cx + radius * angle_rad.cos();
            let y = cy + radius * angle_rad.sin();
            
            // Ensure points stay within canvas bounds
            if (0.0..=100.0).contains(&x) && (0.0..=100.0).contains(&y) {
                draw_point(ctx, x, y, ring_style);
            }
        }
    }
}

/// Check if any endpoint has known latency data
/// 
/// Returns true if at least one endpoint has a latency bucket other than Unknown.
/// Used to conditionally render latency rings only when latency data is available.
/// 
/// Requirements: 1.5
pub fn has_latency_data(endpoints: &[EndpointNode]) -> bool {
    endpoints.iter().any(|node| node.latency_bucket != LatencyBucket::Unknown)
}

/// Calculate endpoint position on the canvas based on latency bucket
/// 
/// Positions endpoints on concentric rings around HOST_CENTER based on their latency:
/// - Low latency (< 50ms): Inner ring (radius 15)
/// - Medium latency (50-200ms): Middle ring (radius 25)
/// - High latency (> 200ms): Outer ring (radius 35)
/// - Unknown latency: Middle ring (radius 25) as fallback
/// 
/// Endpoints are distributed evenly around their assigned ring based on their
/// index within that ring. A small jitter is added to prevent overlap when
/// multiple endpoints share the same ring position.
/// 
/// # Arguments
/// * `endpoint_idx` - Index of this endpoint within its latency bucket group
/// * `total_in_bucket` - Total number of endpoints in the same latency bucket
/// * `latency_bucket` - The latency classification for ring selection
/// 
/// # Returns
/// (x, y) coordinates in virtual canvas space (0-100)
/// 
/// Requirements: 1.2, 1.3, 1.4, 1.5
pub fn calculate_endpoint_position(
    endpoint_idx: usize,
    total_in_bucket: usize,
    latency_bucket: LatencyBucket,
) -> (f64, f64) {
    let (cx, cy) = HOST_CENTER;
    
    // Select ring radius based on latency bucket
    // Unknown latency falls back to middle ring (Requirements 1.5)
    let radius = match latency_bucket {
        LatencyBucket::Low => RING_RADII[0],      // 15.0 - innermost ring
        LatencyBucket::Medium => RING_RADII[1],   // 25.0 - middle ring
        LatencyBucket::High => RING_RADII[2],     // 35.0 - outermost ring
        LatencyBucket::Unknown => RING_RADII[1],  // 25.0 - default to middle ring
    };
    
    // Distribute endpoints evenly around the ring
    // Start from top (-PI/2) and go clockwise
    let total = total_in_bucket.max(1) as f64;
    let angle = (endpoint_idx as f64 / total) * 2.0 * std::f64::consts::PI - std::f64::consts::PI / 2.0;
    
    // Add small jitter to prevent overlap when endpoints are close together
    // Jitter is deterministic based on endpoint index to ensure consistent positioning
    let jitter = ((endpoint_idx % 3) as f64 - 1.0) * 2.0; // -2.0, 0.0, or 2.0
    let effective_radius = radius + jitter;
    
    // Calculate position
    let x = cx + effective_radius * angle.cos();
    let y = cy + effective_radius * angle.sin();
    
    // Clamp to canvas bounds with padding
    (x.clamp(5.0, 95.0), y.clamp(5.0, 95.0))
}

/// Interpolate between two RGB colors based on a ratio (0.0 ~ 1.0)
fn interpolate_color(color1: (u8, u8, u8), color2: (u8, u8, u8), ratio: f32) -> Color {
    let ratio = ratio.clamp(0.0, 1.0);
    let r = (color1.0 as f32 + (color2.0 as f32 - color1.0 as f32) * ratio) as u8;
    let g = (color1.1 as f32 + (color2.1 as f32 - color1.1 as f32) * ratio) as u8;
    let b = (color1.2 as f32 + (color2.2 as f32 - color1.2 as f32) * ratio) as u8;
    Color::Rgb(r, g, b)
}

/// Get color for refresh interval based on its value relative to default
/// 
/// Color coding:
/// - Green (TOXIC_GREEN): Default value (normal performance impact)
/// - Yellow (PUMPKIN_ORANGE): High frequency (increased performance impact)
/// - Red (BLOOD_RED): Very high frequency (significant performance impact)
/// 
/// If recently_changed is true, returns a brighter version of the color
fn get_refresh_color(interval_ms: u64, default_ms: u64, recently_changed: bool) -> Color {
    let base_color = if interval_ms == default_ms {
        // Default value - green
        TOXIC_GREEN
    } else if interval_ms < default_ms {
        // Faster than default (higher frequency)
        let ratio = (default_ms - interval_ms) as f32 / default_ms as f32;
        
        if ratio > 0.5 {
            // Very high frequency - red
            BLOOD_RED
        } else {
            // High frequency - yellow/orange
            PUMPKIN_ORANGE
        }
    } else {
        // Slower than default - also use green (lower resource usage)
        TOXIC_GREEN
    };

    // If recently changed, make the color brighter
    if recently_changed {
        match base_color {
            Color::Rgb(r, g, b) => {
                // Increase brightness by 20%
                let r = ((r as f32 * 1.2).min(255.0)) as u8;
                let g = ((g as f32 * 1.2).min(255.0)) as u8;
                let b = ((b as f32 * 1.2).min(255.0)) as u8;
                Color::Rgb(r, g, b)
            }
            _ => base_color,
        }
    } else {
        base_color
    }
}

// ============================================================================
// Kiroween Overdrive Mode Functions (Requirements 4.2, 4.3, 4.4, 4.5)
// ============================================================================

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

/// Get the normal (non-overdrive) status text for a connection state
/// 
/// Returns standard, professional status descriptions for connection states.
/// Used when Kiroween Overdrive mode is disabled.
/// 
/// # Arguments
/// * `state` - The connection state
/// 
/// # Returns
/// A static string containing the standard status text
pub fn get_normal_status_text(state: ConnectionState) -> &'static str {
    match state {
        ConnectionState::Established => "Alive",
        ConnectionState::Listen => "Listening",
        ConnectionState::TimeWait 
        | ConnectionState::CloseWait 
        | ConnectionState::FinWait1 
        | ConnectionState::FinWait2 
        | ConnectionState::LastAck 
        | ConnectionState::Closing 
        | ConnectionState::Close => "Closing",
        ConnectionState::SynSent | ConnectionState::SynRecv => "Connecting",
        ConnectionState::Unknown => "Unknown",
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

/// Main UI drawing function
pub fn draw(f: &mut Frame, app: &mut AppState) {
    let size = f.area();

    // Main layout: banner, body, status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Banner
            Constraint::Min(0),     // Body
            Constraint::Length(3),  // Status bar
        ])
        .split(size);

    // Banner
    render_banner(f, chunks[0], app);

    // Body: Network map + right panels
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(65), // Network map
            Constraint::Percentage(35), // Right panels
        ])
        .split(chunks[1]);

    render_network_map(f, body_chunks[0], app);
    
    // Right side: Soul Inspector + Grimoire
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Soul Inspector
            Constraint::Percentage(40), // Grimoire
        ])
        .split(body_chunks[1]);
    
    render_soul_inspector(f, right_chunks[0], app);
    render_grimoire(f, right_chunks[1], app);

    // Status bar
    render_status_bar(f, chunks[2], app);
}

fn render_banner(f: &mut Frame, area: Rect, app: &AppState) {
    // Get the appropriate stats label based on overdrive mode (Requirement 4.5)
    // When overdrive is enabled, use "Spirits" instead of "Total Souls"
    let stats_label = get_stats_label(app.graveyard_settings.overdrive_enabled);
    let stats_text = format!("   [💀 {}: 128] [🩸 BPF Radar: ACTIVE]", stats_label);
    
    let banner_text = vec![
        Line::from(vec![
            Span::styled("   _   _  _____  ____   __  __  ____  ", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  | \\ | ||_   _|/ __ \\ |  \\/  ||  _ \\ ", Style::default().fg(Color::Rgb(148, 53, 236))),
            Span::styled("   >>> The Necromancer's Terminal v0.9.0 <<<", Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  |  \\| |  | | | |  | || |\\/| || |_) |", Style::default().fg(Color::Rgb(158, 63, 246))),
            Span::styled("   \"Revealing the unseen connections of the undead.\"", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("  | |\\  |  | | | |__| || |  | || |_ < ", Style::default().fg(Color::Rgb(168, 73, 255))),
        ]),
        Line::from(vec![
            Span::styled("  |_| \\_|  |_|  \\____/ |_|  |_||____/ ", Style::default().fg(Color::Rgb(178, 83, 255))),
            Span::styled(stats_text, Style::default().fg(Color::Red)),
        ]),
    ];

    let banner = Paragraph::new(banner_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Rgb(138, 43, 226)))
        )
        .alignment(Alignment::Left);

    f.render_widget(banner, area);
}

/// Endpoint node for canvas rendering
/// Represents a remote endpoint with its visual properties for the network map
pub struct EndpointNode {
    /// Display label (shortened IP address)
    pub label: String,
    /// X coordinate on canvas (0-100 virtual space)
    pub x: f64,
    /// Y coordinate on canvas (0-100 virtual space)
    pub y: f64,
    /// Dominant connection state for this endpoint
    pub state: ConnectionState,
    /// Number of connections to this endpoint
    pub conn_count: usize,
    /// Latency bucket for ring positioning (Requirements 1.1, 1.2, 1.3, 1.4)
    pub latency_bucket: LatencyBucket,
    /// Endpoint type classification for icon and color selection (Requirements 3.1, 3.2, 3.3, 3.5)
    pub endpoint_type: EndpointType,
    /// Whether this endpoint is a heavy talker (top 5 by connection count) (Requirement 3.4)
    pub is_heavy_talker: bool,
}

fn render_network_map(f: &mut Frame, area: Rect, app: &AppState) {
    use crate::app::GraveyardMode;
    
    // Split: summary line + canvas
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    // Filter connections based on GraveyardMode (Requirement 5.2)
    let filtered_connections: Vec<&crate::net::Connection> = match app.graveyard_mode {
        GraveyardMode::Host => {
            // Host mode: Use all connections
            app.connections.iter().collect()
        }
        GraveyardMode::Process => {
            // Process mode: Filter by selected_process_pid
            if let Some(selected_pid) = app.selected_process_pid {
                app.connections
                    .iter()
                    .filter(|conn| conn.pid == Some(selected_pid))
                    .collect()
            } else {
                // No pid selected, show nothing
                Vec::new()
            }
        }
    };

    // Collect endpoint data from filtered connections
    let mut endpoints_map: HashMap<String, Vec<&crate::net::Connection>> = HashMap::new();
    let mut listen_count = 0;

    for conn in &filtered_connections {
        if conn.state == ConnectionState::Listen {
            listen_count += 1;
        } else if conn.remote_addr != "0.0.0.0" {
            endpoints_map
                .entry(conn.remote_addr.clone())
                .or_default()
                .push(conn);
        }
    }

    let endpoint_count = endpoints_map.len();

    // Determine center node label based on mode (Requirement 5.1)
    // Note: The coffin emoji is now rendered as part of the coffin block,
    // so we only include the text label here
    let center_label = match app.graveyard_mode {
        GraveyardMode::Host => "HOST".to_string(),
        GraveyardMode::Process => {
            if let Some(pid) = app.selected_process_pid {
                // Find the process name from the filtered connections
                let process_name = filtered_connections
                    .iter()
                    .find_map(|conn| {
                        if conn.pid == Some(pid) {
                            conn.process_name.clone()
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "unknown".to_string());
                
                // Truncate process name if too long for coffin block display
                let short_name = if process_name.len() > 8 {
                    format!("{}...", &process_name[..5])
                } else {
                    process_name
                };
                format!("{} ({})", short_name, pid)
            } else {
                "HOST".to_string()
            }
        }
    };

    // Summary line
    let summary = Paragraph::new(Line::from(vec![
        Span::styled(" 📊 ", Style::default().fg(NEON_PURPLE)),
        Span::styled(
            format!(
                "Endpoints: {} | Listening: {} | Total: {}",
                endpoint_count, listen_count, filtered_connections.len()
            ),
            Style::default().fg(BONE_WHITE),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(NEON_PURPLE))
            .title(vec![
                Span::styled(
                    "━ 🕸️ The Graveyard (Network Topology) ━",
                    Style::default()
                        .fg(NEON_PURPLE)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
    );
    f.render_widget(summary, chunks[0]);

    // Prepare endpoint nodes with latency-based ring layout
    // Sort by connection count (descending) to show top N endpoints (Requirement 6.3)
    let mut sorted_endpoints: Vec<_> = endpoints_map.iter().collect();
    sorted_endpoints.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    // Use MAX_VISIBLE_ENDPOINTS to limit displayed nodes (Requirements 6.3, 6.4)
    // This prevents canvas overflow and maintains performance with many connections
    let max_nodes = MAX_VISIBLE_ENDPOINTS;
    let latency_config = &app.latency_config;
    
    // Track how many endpoints are hidden for the "... and N more" indicator
    let hidden_endpoint_count = sorted_endpoints.len().saturating_sub(max_nodes);
    
    // First pass: classify all endpoints by latency bucket and collect metadata
    // Also collect connection counts for heavy talker calculation (Requirement 3.4)
    let endpoint_data: Vec<_> = sorted_endpoints
        .iter()
        .take(max_nodes)
        .map(|(addr, conns)| {
            // Determine dominant state
            let state = conns
                .iter()
                .fold(HashMap::new(), |mut acc: HashMap<ConnectionState, usize>, c| {
                    *acc.entry(c.state).or_insert(0) += 1;
                    acc
                })
                .into_iter()
                .max_by_key(|(_, count)| *count)
                .map(|(state, _)| state)
                .unwrap_or(ConnectionState::Unknown);

            // Shorten label
            let label = if addr.len() > 15 {
                format!("{}...", &addr[..12])
            } else {
                (*addr).to_string()
            };

            // Calculate latency bucket for ring positioning (Requirements 1.1, 1.2, 1.3, 1.4)
            // Currently using None as latency data is not yet available from connections
            // Future tasks will add actual latency measurement to Connection struct
            let latency_bucket = classify_latency(None, latency_config);
            
            // Classify endpoint type based on IP address (Requirements 3.1, 3.2, 3.3, 3.5)
            // Check if this is a LISTEN-only socket (remote = 0.0.0.0:0)
            let is_listen_socket = *addr == "0.0.0.0" && conns.iter().all(|c| c.state == ConnectionState::Listen);
            let endpoint_type = classify_endpoint(addr, is_listen_socket);

            (label, state, conns.len(), latency_bucket, endpoint_type)
        })
        .collect();
    
    // Collect all connection counts for heavy talker calculation (Requirement 3.4)
    let all_conn_counts: Vec<usize> = endpoint_data.iter().map(|(_, _, count, _, _)| *count).collect();
    
    // Count endpoints per latency bucket for even distribution
    let mut bucket_counts: HashMap<LatencyBucket, usize> = HashMap::new();
    for (_, _, _, bucket, _) in &endpoint_data {
        *bucket_counts.entry(*bucket).or_insert(0) += 1;
    }
    
    // Track current index within each bucket for positioning
    let mut bucket_indices: HashMap<LatencyBucket, usize> = HashMap::new();
    
    // Second pass: calculate positions using latency-based ring layout
    // Also calculate heavy talker status for each endpoint (Requirement 3.4)
    let nodes: Vec<EndpointNode> = endpoint_data
        .into_iter()
        .map(|(label, state, conn_count, latency_bucket, endpoint_type)| {
            // Get the index within this bucket and total count for this bucket
            let idx_in_bucket = *bucket_indices.entry(latency_bucket).or_insert(0);
            let total_in_bucket = *bucket_counts.get(&latency_bucket).unwrap_or(&1);
            
            // Increment index for next endpoint in same bucket
            *bucket_indices.get_mut(&latency_bucket).unwrap() += 1;
            
            // Calculate position using latency-based ring layout (Requirements 1.2, 1.3, 1.4, 1.5)
            let (x, y) = calculate_endpoint_position(idx_in_bucket, total_in_bucket, latency_bucket);
            
            // Determine if this endpoint is a heavy talker (Requirement 3.4)
            let is_heavy = is_heavy_talker(conn_count, &all_conn_counts);

            EndpointNode {
                label,
                x,
                y,
                state,
                conn_count,
                latency_bucket,
                endpoint_type,
                is_heavy_talker: is_heavy,
            }
        })
        .collect();

    // Pulsing color for animation
    let pulse_color = interpolate_color((138, 43, 226), (187, 154, 247), app.pulse_phase);

    // Capture values for closure
    let is_empty = nodes.is_empty() && filtered_connections.is_empty();
    let graveyard_mode = app.graveyard_mode;
    
    // Check if any endpoint has latency data for conditional ring rendering (Requirement 1.5)
    let should_draw_rings = has_latency_data(&nodes);
    
    // Capture animation settings for edge particle rendering (Requirements 2.4, 2.5)
    let animations_enabled = app.graveyard_settings.animations_enabled;
    let pulse_phase = app.pulse_phase;
    
    // Track edge count for particle rendering optimization (Requirements 6.1, 6.5)
    // When edge count exceeds PARTICLE_REDUCTION_THRESHOLD, reduce particles per edge
    let edge_count = nodes.len();
    
    // Check if animation complexity has been auto-reduced due to performance (Requirement 6.5)
    let animation_reduced = app.animation_reduced;
    
    // Capture labels setting for conditional label rendering (Requirement 3.6)
    let labels_enabled = app.graveyard_settings.labels_enabled;
    
    // Capture overdrive setting for themed icon rendering (Requirements 4.2, 4.3, 4.4)
    let overdrive_enabled = app.graveyard_settings.overdrive_enabled;
    
    // Capture canvas height for responsive coffin sizing
    // Use terminal rows as proxy for canvas height (each row ~= 2 canvas units)
    let canvas_height = (chunks[1].height as f64) * 2.0;

    // Canvas with Braille markers
    let canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(NEON_PURPLE)),
        )
        .marker(Marker::Braille)
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 100.0])
        .paint(move |ctx| {
            let cx = 50.0;
            let cy = 50.0;
            
            // Draw latency rings first (behind everything else) if latency data exists
            // Requirements: 1.1, 1.5, 1.6
            if should_draw_rings {
                draw_latency_rings(ctx, |ctx, x, y, style| {
                    ctx.print(x, y, Span::styled("·", style));
                });
            }

            // Draw connection lines first (behind nodes)
            // Requirements 2.4, 2.5: Draw base line + particles if animations enabled
            
            // Coffin exclusion zone radius - lines start from outside this radius
            // to avoid overlapping with the central coffin block
            let coffin_radius = 8.0;
            
            for node in &nodes {
                let line_color = match node.state {
                    ConnectionState::Established => TOXIC_GREEN,
                    ConnectionState::TimeWait | ConnectionState::CloseWait => PUMPKIN_ORANGE,
                    ConnectionState::SynSent | ConnectionState::SynRecv => Color::Yellow,
                    ConnectionState::Close => BLOOD_RED,
                    _ => pulse_color,
                };

                // Calculate direction vector from center to endpoint
                let dx = node.x - cx;
                let dy = node.y - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                
                // Calculate line start point at coffin boundary (outside exclusion zone)
                // This prevents lines from overlapping with the coffin block
                let (start_x, start_y) = if dist > coffin_radius {
                    let ratio = coffin_radius / dist;
                    (cx + dx * ratio, cy + dy * ratio)
                } else {
                    (cx, cy) // Fallback for very close endpoints
                };

                // Draw base edge line (always visible for graceful degradation)
                // Requirements 2.5, 2.6, 5.4: When animations are disabled, static connection
                // lines remain visible with state-based colors, ensuring no visual information
                // is lost and full readability is maintained.
                ctx.draw(&CanvasLine {
                    x1: start_x,
                    y1: start_y,
                    x2: node.x,
                    y2: node.y,
                    color: line_color,
                });
                
                // Draw particles along edge if animations are enabled (Requirements 2.4, 2.5)
                // When disabled, only the static edge line above is rendered.
                if animations_enabled {
                    // Performance optimization: Skip particles for edges outside visible area
                    // Requirements 6.1, 6.5: Optimize particle rendering for performance
                    // Check if endpoint is within visible canvas bounds (with small margin)
                    let is_visible = node.x >= 0.0 && node.x <= 100.0 
                                  && node.y >= 0.0 && node.y <= 100.0;
                    
                    if !is_visible {
                        continue; // Skip particle rendering for off-screen edges
                    }
                    
                    // Determine particle color based on edge state and latency (Requirement 2.3)
                    // Priority: Warning states > High latency > Healthy > Normal
                    let particle_color = match node.state {
                        // Warning states: Pumpkin Orange
                        ConnectionState::TimeWait | ConnectionState::CloseWait => PUMPKIN_ORANGE,
                        // Healthy/active: Check latency first
                        ConnectionState::Established => {
                            // High latency connections get warning color
                            if node.latency_bucket == LatencyBucket::High {
                                PUMPKIN_ORANGE
                            } else {
                                TOXIC_GREEN
                            }
                        }
                        // Normal/other connections: Neon Purple
                        _ => NEON_PURPLE,
                    };
                    
                    // Performance optimization: Reduce particle count if many edges or auto-reduced
                    // Requirements 6.1, 6.5: When edge count exceeds threshold or frame time is
                    // consistently high, use fewer particles to maintain performance
                    let particle_offsets: &[f32] = if animation_reduced || edge_count > PARTICLE_REDUCTION_THRESHOLD {
                        &REDUCED_PARTICLE_OFFSETS
                    } else {
                        &PARTICLE_OFFSETS
                    };
                    
                    // Draw particles along the edge (starting from coffin boundary)
                    for &offset in particle_offsets {
                        let (px, py) = particle_position(
                            (start_x, start_y),
                            (node.x, node.y),
                            pulse_phase,
                            offset,
                        );
                        ctx.print(
                            px,
                            py,
                            Span::styled(PARTICLE_SYMBOL, Style::default().fg(particle_color)),
                        );
                    }
                }
            }

            // Draw coffin block at the central HOST node
            // The coffin provides a decorative focal point that enhances the necromancer theme
            // In overdrive mode, the coffin appears in Pumpkin Orange ("burning" effect)
            // The host name is displayed as part of the coffin block (Requirement 5.1)
            // Uses large hexagonal coffin for big screens, mini coffin for small screens
            draw_coffin_block(ctx, &center_label, overdrive_enabled, canvas_height);

            // Draw endpoint nodes with type-specific icons and colors (Requirements 3.1, 3.2, 3.3, 3.4, 3.5)
            // When overdrive is enabled, use themed icons (Requirements 4.2, 4.3, 4.4)
            for node in &nodes {
                // Determine icon based on overdrive mode
                // Requirements 4.2, 4.3, 4.4: Use overdrive icons when enabled
                // Requirements 3.1, 3.2, 3.3, 3.5: Use endpoint type icons when disabled
                let icon = if overdrive_enabled {
                    // Overdrive mode: use state/latency-based themed icons
                    let overdrive_icon = get_overdrive_icon(node.state, node.latency_bucket);
                    // Add heavy talker badge if applicable (Requirement 3.4)
                    if node.is_heavy_talker {
                        format!("{}👑", overdrive_icon)
                    } else {
                        overdrive_icon.to_string()
                    }
                } else {
                    // Normal mode: use endpoint type icon with heavy talker badge
                    node.endpoint_type.icon_with_badge(node.is_heavy_talker)
                };
                
                // Use endpoint_type.color() as base color, but override for warning states
                // This preserves the visual indication of connection issues while still
                // showing endpoint type through the icon
                let color = match node.state {
                    // Warning states override endpoint type color for visibility
                    ConnectionState::TimeWait | ConnectionState::CloseWait => PUMPKIN_ORANGE,
                    ConnectionState::Close => BLOOD_RED,
                    // Use endpoint type color for normal states
                    _ => node.endpoint_type.color(),
                };

                // Node icon (always shown)
                ctx.print(node.x, node.y, Span::styled(icon, Style::default().fg(color)));

                // Node label (IP:port text) - conditionally rendered based on labels_enabled
                // Requirement 3.6: When labels are disabled, show icon only
                if labels_enabled {
                    let label = format!("{} ({})", node.label, node.conn_count);
                    ctx.print(
                        node.x - 6.0,
                        node.y - 4.0,
                        Span::styled(label, Style::default().fg(color)),
                    );
                }
            }

            // Show message if no connections (Requirement 5.3)
            if is_empty {
                let empty_message = match graveyard_mode {
                    GraveyardMode::Process => "(no active connections for this process)",
                    GraveyardMode::Host => "The graveyard is quiet...",
                };
                
                let msg_offset = (empty_message.len() as f64 / 2.0) * 1.2;
                ctx.print(
                    cx - msg_offset,
                    cy - 5.0,
                    Span::styled(
                        empty_message,
                        Style::default().fg(BONE_WHITE).add_modifier(Modifier::ITALIC),
                    ),
                );
            }

            // Show "... and N more" indicator when endpoints exceed MAX_VISIBLE_ENDPOINTS
            // Requirements 6.3, 6.4: Limit visible endpoints and show indicator for hidden ones
            if hidden_endpoint_count > 0 {
                let more_text = format!("... and {} more", hidden_endpoint_count);
                let text_offset = (more_text.len() as f64 / 2.0) * 1.2;
                ctx.print(
                    cx - text_offset,
                    8.0,
                    Span::styled(
                        more_text,
                        Style::default().fg(BONE_WHITE).add_modifier(Modifier::ITALIC),
                    ),
                );
            }
        });

    f.render_widget(canvas, chunks[1]);
}

fn render_soul_inspector(f: &mut Frame, area: Rect, app: &AppState) {
    // Split area for content and sparkline
    let inspector_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11), // Top info with refresh rate
            Constraint::Length(5),  // Sparkline
            Constraint::Min(0),     // Socket list
        ])
        .split(area);

    // Check if refresh interval was recently changed (within CHANGE_HIGHLIGHT_DURATION)
    let recently_changed = app.refresh_config.last_change
        .map(|last| last.elapsed() < crate::app::CHANGE_HIGHLIGHT_DURATION)
        .unwrap_or(false);

    // Get color for refresh interval based on its value
    let refresh_color = get_refresh_color(app.refresh_config.refresh_ms, 100, recently_changed);

    // Apply highlight style if recently changed
    let refresh_style = if recently_changed {
        Style::default().fg(refresh_color).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        Style::default().fg(refresh_color)
    };

    // Get status text based on overdrive mode (Requirement 4.5)
    // When overdrive is enabled, use themed text like "Haunting" instead of "Alive"
    let overdrive_enabled = app.graveyard_settings.overdrive_enabled;
    let status_text = get_status_text(ConnectionState::Established, overdrive_enabled);
    let status_display = format!("🟢 ESTABLISHED ({})", status_text);

    // Top section with process info
    let top_content = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  TARGET: "),
            Span::styled(
                "⚰️ kafka-broker-1",
                Style::default()
                    .fg(PUMPKIN_ORANGE)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("  PID: "),
            Span::styled("4521", Style::default().fg(Color::Cyan)),
            Span::raw("  |  PPID: "),
            Span::styled("1 (init)", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::raw("  USER: "),
            Span::styled("kafka", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("  STATE: "),
            Span::styled(
                status_display,
                Style::default()
                    .fg(TOXIC_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  ⚡ Refresh: "),
            Span::styled(
                format!("{}ms", app.refresh_config.refresh_ms),
                refresh_style,
            ),
        ]),
    ];

    let top_paragraph = Paragraph::new(top_content).block(
        Block::default()
            .title(vec![
                Span::styled(
                    "━ 🔮 Soul Inspector (Detail) ",
                    Style::default()
                        .fg(NEON_PURPLE)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("━━━━━━", Style::default().fg(NEON_PURPLE)),
            ])
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(NEON_PURPLE)),
    );

    f.render_widget(top_paragraph, inspector_chunks[0]);

    // Sparkline for traffic history
    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(vec![Span::styled(
                    " 📊 Traffic History (Last 60s) ",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )])
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(NEON_PURPLE)),
        )
        .data(&app.traffic_history)
        .style(Style::default().fg(TOXIC_GREEN))
        .max(100);

    f.render_widget(sparkline, inspector_chunks[1]);

    // Bottom section with socket list
    let socket_content = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  [📜 Open Sockets List]",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::raw("  > "),
            Span::styled("tcp://0.0.0.0:9092", Style::default().fg(Color::Cyan)),
            Span::styled(" (LISTEN)", Style::default().fg(TOXIC_GREEN)),
        ]),
        Line::from(vec![
            Span::raw("  > "),
            Span::styled("tcp://10.0.1.5:5432", Style::default().fg(Color::Cyan)),
            Span::raw(" -> "),
            Span::styled("db:5432", Style::default().fg(Color::Blue)),
        ]),
        Line::from(vec![
            Span::raw("  > "),
            Span::styled("tcp://[::1]:9093", Style::default().fg(Color::Cyan)),
            Span::styled(" (ESTABLISHED)", Style::default().fg(TOXIC_GREEN)),
        ]),
    ];

    let socket_paragraph = Paragraph::new(socket_content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(NEON_PURPLE)),
    );

    f.render_widget(socket_paragraph, inspector_chunks[2]);
}

fn render_grimoire(f: &mut Frame, area: Rect, app: &mut AppState) {
    use crate::net::ConnectionState;

    let mut log_items = Vec::new();

    // Show all connections (scrollable)
    for (idx, conn) in app.connections.iter().enumerate() {
        // Color based on connection state
        let state_color = match conn.state {
            ConnectionState::Established => TOXIC_GREEN,
            ConnectionState::Listen => BONE_WHITE,
            ConnectionState::TimeWait | ConnectionState::CloseWait => PUMPKIN_ORANGE,
            ConnectionState::Close => BLOOD_RED,
            _ => Color::Gray,
        };

        // Format: local:port -> remote:port [STATE]
        let conn_line = if conn.remote_addr == "0.0.0.0" && conn.remote_port == 0 {
            // Listening socket
            format!(" {}:{} [LISTEN]", conn.local_addr, conn.local_port)
        } else {
            // Active connection
            format!(
                " {}:{} → {}:{} [{:?}]",
                conn.local_addr, conn.local_port, conn.remote_addr, conn.remote_port, conn.state
            )
        };

        // Add process info tag if available (Requirements 6.1, 6.2)
        let process_tag = if let (Some(pid), Some(ref name)) = (conn.pid, &conn.process_name) {
            format!(" [{}({})]", name, pid)
        } else {
            String::new()
        };

        // Check if this connection is selected (Requirement 4.2)
        let is_selected = app.selected_connection == Some(idx);
        
        // Apply highlighting to selected connection
        let item_style = if is_selected {
            Style::default().bg(Color::Rgb(47, 51, 77)) // Deep Indigo background
        } else {
            Style::default()
        };

        log_items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{:2}.", idx + 1), Style::default().fg(Color::DarkGray)),
            Span::styled(conn_line, Style::default().fg(state_color)),
            Span::styled(process_tag, Style::default().fg(Color::Cyan)),
        ])).style(item_style));
    }

    let title = format!("━ 🌐 Active Connections ({}) ", app.connections.len());
    
    let logs = List::new(log_items)
        .block(
            Block::default()
                .title(vec![
                    Span::styled(
                        title,
                        Style::default()
                            .fg(PUMPKIN_ORANGE)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("━━━━━━━", Style::default().fg(PUMPKIN_ORANGE)),
                ])
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(PUMPKIN_ORANGE)),
        )
        .highlight_style(Style::default().bg(Color::Rgb(47, 51, 77)));

    f.render_stateful_widget(logs, area, &mut app.connection_list_state);
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &AppState) {
    use crate::app::GraveyardMode;
    
    // Determine mode-specific hint text
    let mode_hint = match app.graveyard_mode {
        GraveyardMode::Host => "Focus Process | ",
        GraveyardMode::Process => "Back to Host | ",
    };
    
    // Calculate available width for hints (subtract borders and icon)
    let available_width = area.width.saturating_sub(4); // Account for borders and padding
    
    // Define all hints with priority levels (lower number = higher priority)
    // Priority 1: Essential shortcuts (Q, P, arrow keys)
    // Priority 2: Important shortcuts (TAB, refresh controls, toggles)
    // Priority 3: Nice-to-have (F1)
    struct Hint {
        priority: u8,
        key: &'static str,
        desc: String,
        color: Color,
    }
    
    let hints = vec![
        Hint { priority: 1, key: "Q:", desc: "R.I.P ".to_string(), color: Color::Red },
        Hint { priority: 1, key: "↑↓:", desc: "Navigate | ".to_string(), color: NEON_PURPLE },
        Hint { priority: 1, key: "P:", desc: mode_hint.to_string(), color: NEON_PURPLE },
        Hint { priority: 2, key: "+/-:", desc: "Speed | ".to_string(), color: NEON_PURPLE },
        Hint { priority: 2, key: "A:", desc: "Anim | ".to_string(), color: NEON_PURPLE },
        Hint { priority: 2, key: "H:", desc: "Theme | ".to_string(), color: NEON_PURPLE },
        Hint { priority: 2, key: "t:", desc: "Labels | ".to_string(), color: NEON_PURPLE },
        Hint { priority: 3, key: "⇆ TAB:", desc: "Switch Pane | ".to_string(), color: NEON_PURPLE },
        Hint { priority: 3, key: "F1:", desc: "Help | ".to_string(), color: NEON_PURPLE },
    ];
    
    // Build status text, adding hints until we run out of space
    let mut spans = vec![
        Span::styled(" 💀 ", Style::default().fg(NEON_PURPLE)),
    ];
    
    let mut current_length = 4; // Icon + space
    
    // Process hints by priority
    for priority in 1..=3 {
        for hint in &hints {
            if hint.priority == priority {
                let hint_length = hint.key.len() + hint.desc.len();
                if current_length + hint_length <= available_width as usize {
                    spans.push(Span::styled(hint.key, Style::default().fg(hint.color).add_modifier(Modifier::BOLD)));
                    spans.push(Span::raw(hint.desc.clone()));
                    current_length += hint_length;
                }
            }
        }
    }
    
    // Add toggle status indicators (Requirements 5.6)
    // Format: [A:ON/OFF] [H:ON/OFF] [t:ON/OFF]
    // Colors: Toxic Green for ON, Bone White for OFF
    let toggle_indicators = build_toggle_indicators(app);
    let toggle_length: usize = toggle_indicators.iter().map(|s| s.content.len()).sum();
    
    if current_length + toggle_length < available_width as usize {
        spans.push(Span::raw(" "));
        spans.extend(toggle_indicators);
    }
    
    let status_text = Line::from(spans);

    let status_bar = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(NEON_PURPLE))
        )
        .alignment(Alignment::Left);

    f.render_widget(status_bar, area);
}

/// Build toggle status indicator spans for the status bar
/// Shows [A:ON/OFF] [H:ON/OFF] [t:ON/OFF] with appropriate colors
/// Toxic Green for ON, Bone White for OFF (Requirements 5.6)
fn build_toggle_indicators(app: &AppState) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    
    // Animation toggle [A:ON/OFF]
    let anim_state = if app.graveyard_settings.animations_enabled { "ON" } else { "OFF" };
    let anim_color = if app.graveyard_settings.animations_enabled { TOXIC_GREEN } else { BONE_WHITE };
    spans.push(Span::styled("[A:", Style::default().fg(BONE_WHITE)));
    spans.push(Span::styled(anim_state, Style::default().fg(anim_color).add_modifier(Modifier::BOLD)));
    spans.push(Span::styled("] ", Style::default().fg(BONE_WHITE)));
    
    // Overdrive/Theme toggle [H:ON/OFF]
    let overdrive_state = if app.graveyard_settings.overdrive_enabled { "ON" } else { "OFF" };
    let overdrive_color = if app.graveyard_settings.overdrive_enabled { TOXIC_GREEN } else { BONE_WHITE };
    spans.push(Span::styled("[H:", Style::default().fg(BONE_WHITE)));
    spans.push(Span::styled(overdrive_state, Style::default().fg(overdrive_color).add_modifier(Modifier::BOLD)));
    spans.push(Span::styled("] ", Style::default().fg(BONE_WHITE)));
    
    // Labels toggle [t:ON/OFF]
    let labels_state = if app.graveyard_settings.labels_enabled { "ON" } else { "OFF" };
    let labels_color = if app.graveyard_settings.labels_enabled { TOXIC_GREEN } else { BONE_WHITE };
    spans.push(Span::styled("[t:", Style::default().fg(BONE_WHITE)));
    spans.push(Span::styled(labels_state, Style::default().fg(labels_color).add_modifier(Modifier::BOLD)));
    spans.push(Span::styled("]", Style::default().fg(BONE_WHITE)));
    
    spans
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::LatencyConfig;

    // ============================================================================
    // Task 23.1: Test endpoint classification
    // Requirements: 3.1, 3.2, 3.5
    // ============================================================================

    #[test]
    fn test_classify_endpoint_localhost() {
        // Test localhost addresses (Requirement 3.5)
        // 127.0.0.1, ::1, and 0.0.0.0 should all be classified as Localhost
        assert_eq!(classify_endpoint("127.0.0.1", false), EndpointType::Localhost);
        assert_eq!(classify_endpoint("::1", false), EndpointType::Localhost);
        assert_eq!(classify_endpoint("0.0.0.0", false), EndpointType::Localhost);
    }

    #[test]
    fn test_classify_endpoint_rfc1918_class_a() {
        // Test RFC1918 Class A private network: 10.0.0.0/8 (Requirement 3.1)
        assert_eq!(classify_endpoint("10.0.0.1", false), EndpointType::Private);
        assert_eq!(classify_endpoint("10.255.255.255", false), EndpointType::Private);
        assert_eq!(classify_endpoint("10.100.50.25", false), EndpointType::Private);
    }

    #[test]
    fn test_classify_endpoint_rfc1918_class_b() {
        // Test RFC1918 Class B private network: 172.16.0.0/12 (Requirement 3.1)
        // Valid range: 172.16.x.x - 172.31.x.x
        assert_eq!(classify_endpoint("172.16.0.1", false), EndpointType::Private);
        assert_eq!(classify_endpoint("172.31.255.255", false), EndpointType::Private);
        assert_eq!(classify_endpoint("172.20.100.50", false), EndpointType::Private);
        
        // Outside the range should be Public
        assert_eq!(classify_endpoint("172.15.0.1", false), EndpointType::Public);
        assert_eq!(classify_endpoint("172.32.0.1", false), EndpointType::Public);
    }

    #[test]
    fn test_classify_endpoint_rfc1918_class_c() {
        // Test RFC1918 Class C private network: 192.168.0.0/16 (Requirement 3.1)
        assert_eq!(classify_endpoint("192.168.0.1", false), EndpointType::Private);
        assert_eq!(classify_endpoint("192.168.255.255", false), EndpointType::Private);
        assert_eq!(classify_endpoint("192.168.1.100", false), EndpointType::Private);
        
        // Similar but not in range should be Public
        assert_eq!(classify_endpoint("192.169.0.1", false), EndpointType::Public);
        assert_eq!(classify_endpoint("192.167.0.1", false), EndpointType::Public);
    }

    #[test]
    fn test_classify_endpoint_public() {
        // Test public IP addresses (Requirement 3.2)
        // Any IP not in RFC1918 or localhost ranges should be Public
        assert_eq!(classify_endpoint("8.8.8.8", false), EndpointType::Public);
        assert_eq!(classify_endpoint("1.1.1.1", false), EndpointType::Public);
        assert_eq!(classify_endpoint("203.0.113.50", false), EndpointType::Public);
        assert_eq!(classify_endpoint("198.51.100.1", false), EndpointType::Public);
    }

    #[test]
    fn test_classify_endpoint_listen_only() {
        // Test LISTEN-only sockets (Requirement 3.3)
        // When is_listen_socket is true, should return ListenOnly regardless of IP
        assert_eq!(classify_endpoint("0.0.0.0", true), EndpointType::ListenOnly);
        assert_eq!(classify_endpoint("127.0.0.1", true), EndpointType::ListenOnly);
        assert_eq!(classify_endpoint("192.168.1.1", true), EndpointType::ListenOnly);
    }

    #[test]
    fn test_endpoint_type_icons() {
        // Test that each endpoint type returns the correct icon (Requirements 3.1, 3.2, 3.3, 3.5)
        assert_eq!(EndpointType::Localhost.icon(), "⚰️");
        assert_eq!(EndpointType::Private.icon(), "🪦");
        assert_eq!(EndpointType::Public.icon(), "🎃");
        assert_eq!(EndpointType::ListenOnly.icon(), "🕯");
    }

    #[test]
    fn test_endpoint_type_colors() {
        // Test that each endpoint type returns the correct color
        assert_eq!(EndpointType::Localhost.color(), TOXIC_GREEN);
        assert_eq!(EndpointType::Private.color(), BONE_WHITE);
        assert_eq!(EndpointType::Public.color(), PUMPKIN_ORANGE);
        assert_eq!(EndpointType::ListenOnly.color(), NEON_PURPLE);
    }

    #[test]
    fn test_endpoint_type_icon_with_badge() {
        // Test heavy talker badge (Requirement 3.4)
        assert_eq!(EndpointType::Public.icon_with_badge(false), "🎃");
        assert_eq!(EndpointType::Public.icon_with_badge(true), "🎃👑");
        assert_eq!(EndpointType::Private.icon_with_badge(true), "🪦👑");
    }

    // ============================================================================
    // Task 23.2: Test latency bucket classification
    // Requirements: 1.2, 1.3, 1.4, 1.5
    // ============================================================================

    #[test]
    fn test_classify_latency_low() {
        // Test low latency threshold (< 50ms) - Requirement 1.2
        let config = LatencyConfig::default();
        assert_eq!(classify_latency(Some(0), &config), LatencyBucket::Low);
        assert_eq!(classify_latency(Some(25), &config), LatencyBucket::Low);
        assert_eq!(classify_latency(Some(49), &config), LatencyBucket::Low);
    }

    #[test]
    fn test_classify_latency_medium() {
        // Test medium latency range (50-200ms) - Requirement 1.3
        let config = LatencyConfig::default();
        assert_eq!(classify_latency(Some(50), &config), LatencyBucket::Medium);
        assert_eq!(classify_latency(Some(100), &config), LatencyBucket::Medium);
        assert_eq!(classify_latency(Some(200), &config), LatencyBucket::Medium);
    }

    #[test]
    fn test_classify_latency_high() {
        // Test high latency threshold (> 200ms) - Requirement 1.4
        let config = LatencyConfig::default();
        assert_eq!(classify_latency(Some(201), &config), LatencyBucket::High);
        assert_eq!(classify_latency(Some(500), &config), LatencyBucket::High);
        assert_eq!(classify_latency(Some(1000), &config), LatencyBucket::High);
    }

    #[test]
    fn test_classify_latency_unknown() {
        // Test unknown latency (None) handling - Requirement 1.5
        let config = LatencyConfig::default();
        assert_eq!(classify_latency(None, &config), LatencyBucket::Unknown);
    }

    #[test]
    fn test_classify_latency_custom_thresholds() {
        // Test with custom thresholds
        let config = LatencyConfig {
            low_threshold_ms: 100,
            high_threshold_ms: 500,
        };
        
        // Low: < 100ms
        assert_eq!(classify_latency(Some(50), &config), LatencyBucket::Low);
        assert_eq!(classify_latency(Some(99), &config), LatencyBucket::Low);
        
        // Medium: 100-500ms
        assert_eq!(classify_latency(Some(100), &config), LatencyBucket::Medium);
        assert_eq!(classify_latency(Some(300), &config), LatencyBucket::Medium);
        assert_eq!(classify_latency(Some(500), &config), LatencyBucket::Medium);
        
        // High: > 500ms
        assert_eq!(classify_latency(Some(501), &config), LatencyBucket::High);
    }

    // ============================================================================
    // Task 23.3: Test heavy talker detection
    // Requirements: 3.4
    // ============================================================================

    #[test]
    fn test_is_heavy_talker_top_5() {
        // Test that top 5 by connection count are heavy talkers
        let all_counts = vec![100, 80, 60, 40, 20, 10, 5];
        
        // Top 5 should be heavy talkers
        assert!(is_heavy_talker(100, &all_counts)); // 1st
        assert!(is_heavy_talker(80, &all_counts));  // 2nd
        assert!(is_heavy_talker(60, &all_counts));  // 3rd
        assert!(is_heavy_talker(40, &all_counts));  // 4th
        assert!(is_heavy_talker(20, &all_counts));  // 5th
        
        // Below top 5 should not be heavy talkers
        assert!(!is_heavy_talker(10, &all_counts)); // 6th
        assert!(!is_heavy_talker(5, &all_counts));  // 7th
    }

    #[test]
    fn test_is_heavy_talker_fewer_than_5() {
        // Test edge case: fewer than 5 endpoints
        let all_counts = vec![50, 30, 10];
        
        // All should be heavy talkers when < 5 endpoints
        assert!(is_heavy_talker(50, &all_counts));
        assert!(is_heavy_talker(30, &all_counts));
        assert!(is_heavy_talker(10, &all_counts));
    }

    #[test]
    fn test_is_heavy_talker_empty() {
        // Test edge case: empty list
        let all_counts: Vec<usize> = vec![];
        assert!(!is_heavy_talker(10, &all_counts));
    }

    #[test]
    fn test_is_heavy_talker_zero_count() {
        // Test edge case: zero connection count
        let all_counts = vec![10, 5, 0, 0, 0];
        
        // Zero count should not be a heavy talker even if in "top 5"
        assert!(!is_heavy_talker(0, &all_counts));
        assert!(is_heavy_talker(10, &all_counts));
        assert!(is_heavy_talker(5, &all_counts));
    }

    #[test]
    fn test_is_heavy_talker_ties() {
        // Test edge case: ties at the threshold
        let all_counts = vec![100, 50, 50, 50, 50, 10];
        
        // All with count >= 50 should be heavy talkers (ties included)
        assert!(is_heavy_talker(100, &all_counts));
        assert!(is_heavy_talker(50, &all_counts));
        assert!(!is_heavy_talker(10, &all_counts));
    }

    // ============================================================================
    // Task 23.4: Test particle position calculation
    // Requirements: 2.2
    // ============================================================================

    #[test]
    fn test_particle_position_at_start() {
        // Test particle at phase 0.0 (start of edge)
        let start = (50.0, 50.0);
        let end = (80.0, 30.0);
        
        let pos = particle_position(start, end, 0.0, 0.0);
        assert!((pos.0 - 50.0).abs() < 0.001);
        assert!((pos.1 - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_particle_position_at_middle() {
        // Test particle at phase 0.5 (middle of edge)
        let start = (50.0, 50.0);
        let end = (80.0, 30.0);
        
        let pos = particle_position(start, end, 0.5, 0.0);
        // Expected: (65.0, 40.0) - midpoint
        assert!((pos.0 - 65.0).abs() < 0.001);
        assert!((pos.1 - 40.0).abs() < 0.001);
    }

    #[test]
    fn test_particle_position_at_end() {
        // Test particle at phase 1.0 (wraps to start)
        let start = (50.0, 50.0);
        let end = (80.0, 30.0);
        
        let pos = particle_position(start, end, 1.0, 0.0);
        // Phase 1.0 % 1.0 = 0.0, so should be at start
        assert!((pos.0 - 50.0).abs() < 0.001);
        assert!((pos.1 - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_particle_position_with_offset() {
        // Test particle with offset (multiple particles along edge)
        let start = (0.0, 0.0);
        let end = (100.0, 100.0);
        
        // Phase 0.0 + offset 0.33 = position at 33%
        let pos = particle_position(start, end, 0.0, 0.33);
        assert!((pos.0 - 33.0).abs() < 0.001);
        assert!((pos.1 - 33.0).abs() < 0.001);
        
        // Phase 0.0 + offset 0.66 = position at 66%
        let pos = particle_position(start, end, 0.0, 0.66);
        assert!((pos.0 - 66.0).abs() < 0.001);
        assert!((pos.1 - 66.0).abs() < 0.001);
    }

    #[test]
    fn test_particle_position_wrapping() {
        // Test that phase + offset wraps correctly
        let start = (0.0, 0.0);
        let end = (100.0, 0.0);
        
        // Phase 0.8 + offset 0.33 = 1.13 % 1.0 = 0.13
        let pos = particle_position(start, end, 0.8, 0.33);
        let expected_t = (0.8 + 0.33) % 1.0; // 0.13
        assert!((pos.0 - expected_t * 100.0).abs() < 0.001);
    }

    // ============================================================================
    // Additional tests for overdrive mode functions
    // Requirements: 4.2, 4.3, 4.4, 4.5
    // ============================================================================

    #[test]
    fn test_get_overdrive_icon() {
        use crate::net::ConnectionState;
        
        // Test ESTABLISHED with normal latency (Requirement 4.2)
        assert_eq!(get_overdrive_icon(ConnectionState::Established, LatencyBucket::Low), "🟢👻");
        assert_eq!(get_overdrive_icon(ConnectionState::Established, LatencyBucket::Medium), "🟢👻");
        
        // Test high latency (Requirement 4.3)
        assert_eq!(get_overdrive_icon(ConnectionState::Established, LatencyBucket::High), "🔥🎃");
        
        // Test CLOSE_WAIT/TIME_WAIT (Requirement 4.4)
        assert_eq!(get_overdrive_icon(ConnectionState::CloseWait, LatencyBucket::Low), "💀");
        assert_eq!(get_overdrive_icon(ConnectionState::TimeWait, LatencyBucket::Medium), "💀");
    }

    #[test]
    fn test_get_overdrive_status_text() {
        use crate::net::ConnectionState;
        
        // Test status text transformations (Requirement 4.5)
        assert_eq!(get_overdrive_status_text(ConnectionState::Established), "Haunting");
        assert_eq!(get_overdrive_status_text(ConnectionState::Listen), "Summoning");
        assert_eq!(get_overdrive_status_text(ConnectionState::TimeWait), "Fading");
        assert_eq!(get_overdrive_status_text(ConnectionState::CloseWait), "Fading");
    }

    #[test]
    fn test_get_stats_label() {
        // Test stats label based on overdrive mode (Requirement 4.5)
        assert_eq!(get_stats_label(false), "Connections");
        assert_eq!(get_stats_label(true), "Spirits");
    }

    #[test]
    fn test_get_status_text() {
        use crate::net::ConnectionState;
        
        // Test normal mode
        assert_eq!(get_status_text(ConnectionState::Established, false), "Alive");
        assert_eq!(get_status_text(ConnectionState::Listen, false), "Listening");
        
        // Test overdrive mode
        assert_eq!(get_status_text(ConnectionState::Established, true), "Haunting");
        assert_eq!(get_status_text(ConnectionState::Listen, true), "Summoning");
    }

    // ============================================================================
    // Test endpoint position calculation
    // Requirements: 1.2, 1.3, 1.4, 1.5
    // ============================================================================

    #[test]
    fn test_calculate_endpoint_position_ring_selection() {
        // Test that different latency buckets map to different rings
        let (x_low, y_low) = calculate_endpoint_position(0, 1, LatencyBucket::Low);
        let (x_med, y_med) = calculate_endpoint_position(0, 1, LatencyBucket::Medium);
        let (x_high, y_high) = calculate_endpoint_position(0, 1, LatencyBucket::High);
        
        // Calculate distances from center (50, 50)
        let dist_low = ((x_low - 50.0).powi(2) + (y_low - 50.0).powi(2)).sqrt();
        let dist_med = ((x_med - 50.0).powi(2) + (y_med - 50.0).powi(2)).sqrt();
        let dist_high = ((x_high - 50.0).powi(2) + (y_high - 50.0).powi(2)).sqrt();
        
        // Low latency should be closest, high latency should be farthest
        assert!(dist_low < dist_med);
        assert!(dist_med < dist_high);
    }

    #[test]
    fn test_calculate_endpoint_position_unknown_fallback() {
        // Test that Unknown latency falls back to middle ring (Requirement 1.5)
        let (x_unknown, y_unknown) = calculate_endpoint_position(0, 1, LatencyBucket::Unknown);
        let (x_medium, y_medium) = calculate_endpoint_position(0, 1, LatencyBucket::Medium);
        
        // Unknown should use same ring as Medium
        let dist_unknown = ((x_unknown - 50.0).powi(2) + (y_unknown - 50.0).powi(2)).sqrt();
        let dist_medium = ((x_medium - 50.0).powi(2) + (y_medium - 50.0).powi(2)).sqrt();
        
        // Should be approximately equal (within jitter range)
        assert!((dist_unknown - dist_medium).abs() < 5.0);
    }

    #[test]
    fn test_calculate_endpoint_position_bounds() {
        // Test that positions are clamped within canvas bounds
        for i in 0..10 {
            for bucket in [LatencyBucket::Low, LatencyBucket::Medium, LatencyBucket::High] {
                let (x, y) = calculate_endpoint_position(i, 10, bucket);
                assert!(x >= 5.0 && x <= 95.0, "x={} out of bounds", x);
                assert!(y >= 5.0 && y <= 95.0, "y={} out of bounds", y);
            }
        }
    }

    #[test]
    fn test_has_latency_data() {
        // Test has_latency_data function (Requirement 1.5)
        let nodes_with_data = vec![
            EndpointNode {
                label: "test".to_string(),
                x: 50.0,
                y: 50.0,
                state: crate::net::ConnectionState::Established,
                conn_count: 1,
                latency_bucket: LatencyBucket::Low,
                endpoint_type: EndpointType::Public,
                is_heavy_talker: false,
            },
        ];
        assert!(has_latency_data(&nodes_with_data));
        
        let nodes_without_data = vec![
            EndpointNode {
                label: "test".to_string(),
                x: 50.0,
                y: 50.0,
                state: crate::net::ConnectionState::Established,
                conn_count: 1,
                latency_bucket: LatencyBucket::Unknown,
                endpoint_type: EndpointType::Public,
                is_heavy_talker: false,
            },
        ];
        assert!(!has_latency_data(&nodes_without_data));
        
        // Empty list should return false
        let empty_nodes: Vec<EndpointNode> = vec![];
        assert!(!has_latency_data(&empty_nodes));
    }
}
