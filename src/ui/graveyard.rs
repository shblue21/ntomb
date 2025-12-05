// Graveyard (Network Map) rendering module
//
// Renders the main network topology visualization canvas with endpoints,
// connections, latency rings, and particle animations.

use crate::app::{AppState, GraveyardMode, LatencyBucket, LatencyConfig};
use crate::net::ConnectionState;
use crate::theme::{
    get_overdrive_icon, interpolate_color, BLOOD_RED, BONE_WHITE, NEON_PURPLE, PUMPKIN_ORANGE,
    TOXIC_GREEN,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Line as CanvasLine},
        Block, BorderType, Borders, Paragraph,
    },
    Frame,
};
use std::collections::HashMap;

// Latency ring constants for Graveyard visualization (Requirements 1.1, 1.6)
// Ring radii in virtual canvas space (0-100)
// Inner ring (Low latency < 50ms), Middle ring (Medium 50-200ms), Outer ring (High > 200ms)
// INCREASED to utilize more canvas space and keep nodes away from coffin
const RING_RADII: [f64; 3] = [25.0, 35.0, 45.0];

// ============================================================================
// Layout Constants (Requirements 1.1, 1.4)
// ============================================================================

/// Minimum edge padding in canvas units
/// Ensures labels have space even on small canvases
/// Requirements: 1.4
const MIN_EDGE_PADDING: f64 = 5.0;

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
// Limited to 8 for clean visualization around the central HOST
const MAX_VISIBLE_ENDPOINTS: usize = 8;

// Threshold for reducing particle count to maintain performance
// When edge count exceeds this, reduce particles per edge
const PARTICLE_REDUCTION_THRESHOLD: usize = 50;

// Reduced particle offsets for high edge count scenarios
// Uses 1 particle instead of 3 to reduce rendering load
const REDUCED_PARTICLE_OFFSETS: [f32; 1] = [0.33];

// ============================================================================
// Adaptive Layout Configuration (Requirements 1.1, 1.2, 2.1)
// ============================================================================

/// Layout configuration calculated from canvas dimensions
///
/// Contains the ring radii and other layout parameters that adapt
/// to the available canvas space. When the canvas is large enough,
/// rings scale proportionally to utilize the space. On smaller screens,
/// fixed radii are used to maintain readability.
///
/// Requirements: 1.1, 1.2, 1.3, 2.1
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutConfig {
    /// Radius for Low latency ring (innermost)
    pub ring_low: f64,
    /// Radius for Medium latency ring (middle)
    pub ring_medium: f64,
    /// Radius for High latency ring (outermost)
    pub ring_high: f64,
    /// Minimum padding from canvas edges
    pub edge_padding: f64,
    /// Whether adaptive mode is active (vs fixed fallback)
    pub is_adaptive: bool,
}

impl Default for LayoutConfig {
    /// Returns the default fixed layout configuration
    ///
    /// Uses the default ring radii (15, 25, 35) with minimum edge padding.
    /// This is the fallback for small canvases.
    fn default() -> Self {
        Self {
            ring_low: RING_RADII[0],
            ring_medium: RING_RADII[1],
            ring_high: RING_RADII[2],
            edge_padding: MIN_EDGE_PADDING,
            is_adaptive: false,
        }
    }
}



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

// ============================================================================
// Classic Coffin Rendering System (Requirements 3.1)
// HARDCODED TEMPLATES - DO NOT MODIFY THE ASCII ART
// ============================================================================
//
// ┌─────────────────────────────────────────────────────────────────────────┐
// │ WARNING: The coffin ASCII art templates below are DESIGN ARTIFACTS.     │
// │ DO NOT modify the shape, characters, or line structure.                 │
// │ Only the HOST placeholder may be replaced with actual host names.       │
// │ Tests in this module will FAIL if the templates are changed.            │
// └─────────────────────────────────────────────────────────────────────────┘

/// Large coffin template (4 lines) - FIXED ASCII ART, DO NOT CHANGE
///
/// Visual representation (with 6-char HOST placeholder):
/// ```text
///    /‾‾‾‾‾‾\
///   / HOST__ \
///   \        /
///    \______/
/// ```
///
/// Template rules:
/// - Line 0: Top point of coffin (14 chars)
/// - Line 1: HOST__ placeholder (6 chars, replaced with actual hostname) (14 chars)
/// - Line 2: Lower body widening (14 chars)
/// - Line 3: Bottom base (14 chars)
/// - Total width: 14 characters per line (ALL LINES MUST BE EXACTLY 14 CHARS)
const LARGE_COFFIN_TEMPLATE: [&str; 4] = [
    "   /‾‾‾‾‾‾\\   ",  // Line 0: 14 chars (3 + / + 6 + \ + 3)
    "  / HOST__ \\  ",   // Line 1: 14 chars (2 + / + 1 + HOST__ + 1 + \ + 2)
    "  \\        /  ",   // Line 2: 14 chars (2 + \ + 8 + / + 2)
    "   \\______/   ",   // Line 3: 14 chars (3 + \ + 6 + / + 3)
];

/// Large coffin dimensions (characters)
const LARGE_COFFIN_WIDTH: usize = 14;
const LARGE_COFFIN_HEIGHT: usize = 4;
/// Maximum host name length that fits in large coffin
const LARGE_COFFIN_MAX_NAME: usize = 6;
/// Placeholder string in template (exactly 6 chars)
const LARGE_COFFIN_PLACEHOLDER: &str = "HOST__";

/// Mid coffin template (3 lines) - FIXED ASCII ART, DO NOT CHANGE
///
/// Visual representation (with 6-char HOST placeholder):
/// ```text
///  /‾‾‾‾‾‾\
/// / HOST__ \
///  \______/
/// ```
///
/// Template rules:
/// - Line 0: Top (11 chars)
/// - Line 1: HOST__ placeholder (6 chars, replaced with actual hostname) (11 chars)
/// - Line 2: Bottom base (11 chars)
/// - Total width: 11 characters per line (ALL LINES MUST BE EXACTLY 11 CHARS)
const MID_COFFIN_TEMPLATE: [&str; 3] = [
    " /‾‾‾‾‾‾\\  ",    // Line 0: 11 chars (1 + / + 6 macrons + \ + 2)
    "/ HOST__ \\ ",    // Line 1: 11 chars (/ + 1 + HOST__ + 1 + \ + 1)
    " \\______/  ",    // Line 2: 11 chars (1 + \ + 6 + / + 2)
];

/// Mid coffin dimensions (characters)
const MID_COFFIN_WIDTH: usize = 11;
const MID_COFFIN_HEIGHT: usize = 3;
/// Maximum host name length that fits in mid coffin
const MID_COFFIN_MAX_NAME: usize = 6;
/// Placeholder string in template (exactly 6 chars)
const MID_COFFIN_PLACEHOLDER: &str = "HOST__";

/// Coffin variant enumeration
///
/// Determines which coffin template to use based on available space.
/// The system gracefully degrades from Large → Mid → Label as space decreases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoffinVariant {
    /// 5-line full coffin silhouette (requires 12x5 character space)
    Large,
    /// 3-line compact coffin (requires 10x3 character space)
    Mid,
    /// 1-line label fallback: [⚰ HOST]
    Label,
}

/// Coffin rendering result
///
/// Contains the pre-rendered coffin lines and metadata for positioning.
/// The `lines` vector contains the exact strings to render, with HOST
/// already replaced by the actual hostname.
#[derive(Debug, Clone)]
pub struct CoffinRender {
    /// Coffin lines from top to bottom (ready to render)
    pub lines: Vec<String>,
    /// Which variant was selected
    pub variant: CoffinVariant,
    /// Maximum width in characters (for centering calculations)
    pub width: usize,
    /// Height in lines
    pub height: usize,
}



/// Truncate host name to fit within max_len, adding ".." suffix if needed
///
/// # Examples
/// - "HOST" with max_len=10 → "HOST"
/// - "kafka-broker-1" with max_len=6 → "kafk.."
/// - "AB" with max_len=2 → "AB"
fn truncate_host_name(host: &str, max_len: usize) -> String {
    let char_count = host.chars().count();
    if char_count <= max_len {
        host.to_string()
    } else if max_len <= 3 {
        // Too short for suffix, just truncate
        host.chars().take(max_len).collect()
    } else {
        // Truncate and add ".." suffix
        let truncated: String = host.chars().take(max_len - 2).collect();
        format!("{}..", truncated)
    }
}

/// Center-pad a string to fit within a given width
///
/// # Examples
/// - "X" with width=5 → "  X  "
/// - "HOST" with width=6 → " HOST "
fn center_pad(s: &str, width: usize) -> String {
    let s_len = s.chars().count();
    if s_len >= width {
        return s.to_string();
    }
    let total_pad = width - s_len;
    let left_pad = total_pad / 2;
    let right_pad = total_pad - left_pad;
    format!("{}{}{}", " ".repeat(left_pad), s, " ".repeat(right_pad))
}

/// Build Large coffin (5 lines) from hardcoded template
///
/// Replaces the "HOST__" placeholder in line 2 with the actual hostname.
/// The hostname is truncated to LARGE_COFFIN_MAX_NAME chars and centered.
///
/// # Arguments
/// * `host` - The hostname to display (e.g., "HOST", "kafka-broker-1")
///
/// # Returns
/// CoffinRender with 5 lines ready to display
pub fn build_large_coffin(host: &str) -> CoffinRender {
    let display_name = truncate_host_name(host, LARGE_COFFIN_MAX_NAME);
    let padded_name = center_pad(&display_name, LARGE_COFFIN_MAX_NAME);
    
    // Replace "HOST__" placeholder (6 chars) with padded hostname (6 chars)
    // This maintains exact line width
    let lines: Vec<String> = LARGE_COFFIN_TEMPLATE
        .iter()
        .map(|line| {
            if line.contains(LARGE_COFFIN_PLACEHOLDER) {
                line.replace(LARGE_COFFIN_PLACEHOLDER, &padded_name)
            } else {
                line.to_string()
            }
        })
        .collect();
    
    CoffinRender {
        lines,
        variant: CoffinVariant::Large,
        width: LARGE_COFFIN_WIDTH,
        height: LARGE_COFFIN_HEIGHT,
    }
}

/// Build Mid coffin (3 lines) from hardcoded template
///
/// Replaces the "HOST__" placeholder in line 1 with the actual hostname.
/// The hostname is truncated to MID_COFFIN_MAX_NAME chars and centered.
///
/// # Arguments
/// * `host` - The hostname to display
///
/// # Returns
/// CoffinRender with 3 lines ready to display
pub fn build_mid_coffin(host: &str) -> CoffinRender {
    let display_name = truncate_host_name(host, MID_COFFIN_MAX_NAME);
    let padded_name = center_pad(&display_name, MID_COFFIN_MAX_NAME);
    
    // Replace "HOST__" placeholder (6 chars) with padded hostname (6 chars)
    // This maintains exact line width
    let lines: Vec<String> = MID_COFFIN_TEMPLATE
        .iter()
        .map(|line| {
            if line.contains(MID_COFFIN_PLACEHOLDER) {
                line.replace(MID_COFFIN_PLACEHOLDER, &padded_name)
            } else {
                line.to_string()
            }
        })
        .collect();
    
    CoffinRender {
        lines,
        variant: CoffinVariant::Mid,
        width: MID_COFFIN_WIDTH,
        height: MID_COFFIN_HEIGHT,
    }
}

/// Build Label coffin (1 line) - minimal fallback
///
/// Format: [⚰ HOST]
/// Used when there's not enough space for even the Mid coffin.
///
/// # Arguments
/// * `host` - The hostname to display
/// * `max_width` - Maximum available width in characters
///
/// # Returns
/// CoffinRender with 1 line in format "[⚰ {hostname}]"
pub fn build_label_coffin(host: &str, max_width: usize) -> CoffinRender {
    // Reserve space for "[⚰ " (3 chars) and "]" (1 char) = 4 chars total
    // Note: ⚰ is a single-width character in most terminals
    let available = max_width.saturating_sub(4);
    let display_name = truncate_host_name(host, available.max(3));
    let line = format!("[⚰ {}]", display_name);
    let width = line.chars().count();
    
    CoffinRender {
        lines: vec![line],
        variant: CoffinVariant::Label,
        width,
        height: 1,
    }
}

/// Choose the appropriate coffin variant based on available area
///
/// Selection logic (graceful degradation):
/// 1. If area fits Large coffin (14 chars wide, 5 lines tall) → use Large
/// 2. Else if area fits Mid coffin (11 chars wide, 3 lines tall) → use Mid
/// 3. Else → use Label (1 line fallback)
///
/// The coffin is NEVER partially rendered. Either the full variant fits,
/// or we degrade to a smaller variant.
///
/// # Arguments
/// * `area_width` - Available width in canvas units (0-100 scale)
/// * `area_height` - Available height in canvas units (0-100 scale)
/// * `host` - Host name to display
///
/// # Returns
/// CoffinRender with the largest variant that fits completely
///
/// # Canvas-to-Character Conversion
/// - Width: 1 canvas unit ≈ 1 character
/// - Height: 4 canvas units ≈ 1 line (due to terminal aspect ratio)
pub fn choose_coffin_variant(area_width: f64, area_height: f64, host: &str) -> CoffinRender {
    // Convert canvas units to approximate character dimensions
    // Terminal cells are typically ~2:1 aspect ratio (taller than wide)
    let char_width = (area_width / 1.0) as usize;
    let char_height = (area_height / 4.0) as usize;
    
    // Try Large coffin first (5 lines, 14 chars wide)
    // Requires: width >= 14, height >= 5
    if char_width >= LARGE_COFFIN_WIDTH && char_height >= LARGE_COFFIN_HEIGHT {
        return build_large_coffin(host);
    }
    
    // Try Mid coffin (3 lines, 11 chars wide)
    // Requires: width >= 11, height >= 3
    if char_width >= MID_COFFIN_WIDTH && char_height >= MID_COFFIN_HEIGHT {
        return build_mid_coffin(host);
    }
    
    // Fallback to Label (1 line, minimum 10 chars for readability)
    build_label_coffin(host, char_width.max(10))
}

/// Calculate the coffin exclusion zone radius
///
/// Returns the radius (in canvas units) around HOST_CENTER where
/// connection lines should not be drawn to avoid overlapping the coffin.
///
/// # Arguments
/// * `variant` - The coffin variant being rendered
///
/// # Returns
/// Radius in canvas units for the exclusion zone
pub fn coffin_exclusion_radius(variant: CoffinVariant) -> f64 {
    match variant {
        CoffinVariant::Large => 15.0,  // 5-line coffin needs larger exclusion
        CoffinVariant::Mid => 12.0,    // 3-line coffin
        CoffinVariant::Label => 8.0,   // 1-line label
    }
}

/// Draw the coffin block on the canvas at the HOST center
///
/// Renders a classic coffin silhouette for the central HOST node.
/// Automatically selects the appropriate variant based on canvas size.
///
/// The coffin is drawn AFTER the connection lines and particles,
/// ensuring it appears on top and is never obscured.
///
/// # Arguments
/// * `ctx` - The canvas context for drawing
/// * `host_name` - The name to display (e.g., "HOST", "kafka-broker-1")
/// * `overdrive_enabled` - When true, uses Pumpkin Orange for a "burning" effect
/// * `canvas_height` - Height of the canvas in canvas units
/// * `center_x` - X coordinate of the center point (for aspect-ratio adjusted canvases)
/// * `center_y` - Y coordinate of the center point (typically 50.0)
///
/// # Returns
/// The CoffinVariant that was rendered (for exclusion zone calculation)
pub fn draw_coffin_block(
    ctx: &mut ratatui::widgets::canvas::Context<'_>,
    host_name: &str,
    overdrive_enabled: bool,
    canvas_height: f64,
    center_x: f64,
    center_y: f64,
) -> CoffinVariant {
    let (cx, cy) = (center_x, center_y);
    
    // Coffin color: Neon Purple normally, Pumpkin Orange in overdrive mode
    let coffin_color = if overdrive_enabled {
        PUMPKIN_ORANGE
    } else {
        NEON_PURPLE
    };
    
    // Choose coffin variant based on canvas size (100x100 virtual space)
    let coffin = choose_coffin_variant(100.0, canvas_height, host_name);
    let variant = coffin.variant;
    
    let style = Style::default().fg(coffin_color).add_modifier(Modifier::BOLD);
    
    // Calculate vertical spacing based on variant
    // Larger spacing for larger coffins to maintain proportions
    let line_spacing = match coffin.variant {
        CoffinVariant::Large => 4.0,
        CoffinVariant::Mid => 4.5,
        CoffinVariant::Label => 0.0,
    };
    
    // Calculate starting Y position (center the coffin vertically)
    let total_height = (coffin.height as f64 - 1.0) * line_spacing;
    let start_y = cy + total_height / 2.0;
    
    // Cell width for horizontal positioning (1 canvas unit per character)
    let cell_width = 1.0;
    
    // Use the coffin's fixed width for centering (not line.chars().count())
    // This ensures consistent centering regardless of Unicode character widths
    let coffin_width = coffin.width as f64 * cell_width;
    
    // Draw each line of the coffin from top to bottom
    for (i, line) in coffin.lines.iter().enumerate() {
        // Center based on fixed coffin width, not individual line width
        let x = cx - coffin_width / 2.0;
        let y = start_y - (i as f64 * line_spacing);
        
        ctx.print(x, y, Span::styled(line.clone(), style));
    }
    
    variant
}

/// Get the coffin variant that would be selected for given canvas dimensions
///
/// This is useful for pre-calculating the exclusion zone before drawing.
///
/// # Arguments
/// * `canvas_height` - Height of the canvas in canvas units
/// * `host_name` - The hostname (affects nothing, but needed for API consistency)
///
/// # Returns
/// The CoffinVariant that would be selected
pub fn get_coffin_variant_for_canvas(canvas_height: f64, host_name: &str) -> CoffinVariant {
    choose_coffin_variant(100.0, canvas_height, host_name).variant
}

/// Draw latency rings on the canvas around the HOST center
/// 
/// Draws 3 concentric dotted circles using Braille markers:
/// - Inner ring: Low latency endpoints (< 50ms)
/// - Middle ring: Medium latency endpoints (50-200ms)
/// - Outer ring: High latency endpoints (> 200ms)
/// 
/// Ring radii are determined by the provided LayoutConfig, enabling adaptive
/// scaling based on canvas dimensions.
/// 
/// Requirements: 1.1, 2.1
pub fn draw_latency_rings<F>(
    ctx: &mut ratatui::widgets::canvas::Context<'_>,
    layout: &LayoutConfig,
    draw_point: F,
)
where
    F: Fn(&mut ratatui::widgets::canvas::Context<'_>, f64, f64, Style),
{
    let (cx, cy) = HOST_CENTER;
    
    // Use adaptive ring radii from layout config
    let ring_radii = [layout.ring_low, layout.ring_medium, layout.ring_high];
    
    for (ring_idx, radius) in ring_radii.iter().enumerate() {
        // Calculate opacity: inner ring is brightest, outer rings fade
        let opacity_factor = 1.0 - (ring_idx as f32 * 0.25);
        let r = (169.0 * opacity_factor) as u8;
        let g = (177.0 * opacity_factor) as u8;
        let b = (214.0 * opacity_factor) as u8;
        let ring_color = Color::Rgb(r, g, b);
        let ring_style = Style::default().fg(ring_color);
        
        // Draw ring as series of dotted points (every 10 degrees for dotted effect)
        for angle_deg in (0..360).step_by(10) {
            let angle_rad = (angle_deg as f64).to_radians();
            let x = cx + radius * angle_rad.cos();
            let y = cy + radius * angle_rad.sin();
            
            // Ensure points stay within canvas bounds with padding
            let min_bound = layout.edge_padding;
            let max_bound = 100.0 - layout.edge_padding;
            if x >= min_bound && x <= max_bound && y >= min_bound && y <= max_bound {
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
/// Positions endpoints on concentric rings around HOST_CENTER based on their latency.
/// Uses the provided LayoutConfig to determine ring radii, enabling adaptive scaling
/// based on canvas dimensions.
/// 
/// # Arguments
/// * `endpoint_idx` - Index of this endpoint within its latency bucket
/// * `total_in_bucket` - Total number of endpoints in the same bucket
/// * `latency_bucket` - The latency classification for ring selection
/// * `layout` - Layout configuration with calculated ring radii
/// 
/// # Returns
/// (x, y) coordinates in canvas space, clamped to stay within bounds
/// 
/// Requirements: 1.2, 2.1, 2.3
pub fn calculate_endpoint_position(
    endpoint_idx: usize,
    total_in_bucket: usize,
    latency_bucket: LatencyBucket,
    layout: &LayoutConfig,
) -> (f64, f64) {
    let (cx, cy) = HOST_CENTER;
    
    // Select ring radius based on latency bucket using adaptive layout config
    let radius = match latency_bucket {
        LatencyBucket::Low => layout.ring_low,
        LatencyBucket::Medium => layout.ring_medium,
        LatencyBucket::High => layout.ring_high,
        LatencyBucket::Unknown => layout.ring_medium, // Default to medium ring
    };
    
    // Distribute endpoints evenly around the ring
    let total = total_in_bucket.max(1) as f64;
    let angle = (endpoint_idx as f64 / total) * 2.0 * std::f64::consts::PI - std::f64::consts::PI / 2.0;
    
    // Add small jitter to prevent overlap
    let jitter = ((endpoint_idx % 3) as f64 - 1.0) * 2.0;
    let effective_radius = radius + jitter;
    
    // Calculate position
    let x = cx + effective_radius * angle.cos();
    let y = cy + effective_radius * angle.sin();
    
    // Clamp to canvas bounds with padding from layout config
    let min_bound = layout.edge_padding;
    let max_bound = 100.0 - layout.edge_padding;
    (x.clamp(min_bound, max_bound), y.clamp(min_bound, max_bound))
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
    /// Latency bucket for ring positioning
    pub latency_bucket: LatencyBucket,
    /// Endpoint type classification for icon and color selection
    pub endpoint_type: EndpointType,
    /// Whether this endpoint is a heavy talker (top 5 by connection count)
    pub is_heavy_talker: bool,
}

pub fn render_network_map(f: &mut Frame, area: Rect, app: &AppState) {
    // Split: summary line + canvas
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    // Filter connections based on GraveyardMode
    let filtered_connections: Vec<&crate::net::Connection> = match app.graveyard_mode {
        GraveyardMode::Host => {
            app.connections.iter().collect()
        }
        GraveyardMode::Process => {
            if let Some(selected_pid) = app.selected_process_pid {
                app.connections
                    .iter()
                    .filter(|conn| conn.pid == Some(selected_pid))
                    .collect()
            } else {
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

    // Determine center node label based on mode
    let center_label = match app.graveyard_mode {
        GraveyardMode::Host => "HOST".to_string(),
        GraveyardMode::Process => {
            if let Some(pid) = app.selected_process_pid {
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

    // Summary line with legend
    let summary = Paragraph::new(Line::from(vec![
        Span::styled(" 📊 ", Style::default().fg(NEON_PURPLE)),
        Span::styled(
            format!(
                "Endpoints: {} | Listening: {} | Total: {}  ",
                endpoint_count, listen_count, filtered_connections.len()
            ),
            Style::default().fg(BONE_WHITE),
        ),
        // Legend for icons
        Span::styled("[", Style::default().fg(Color::DarkGray)),
        Span::styled("⚰️", Style::default().fg(PUMPKIN_ORANGE)),
        Span::styled("host ", Style::default().fg(Color::DarkGray)),
        Span::styled("🏠", Style::default().fg(TOXIC_GREEN)),
        Span::styled("local ", Style::default().fg(Color::DarkGray)),
        Span::styled("🎃", Style::default().fg(PUMPKIN_ORANGE)),
        Span::styled("ext ", Style::default().fg(Color::DarkGray)),
        Span::styled("👑", Style::default().fg(Color::Yellow)),
        Span::styled("hot", Style::default().fg(Color::DarkGray)),
        Span::styled("]", Style::default().fg(Color::DarkGray)),
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
    let mut sorted_endpoints: Vec<_> = endpoints_map.iter().collect();
    sorted_endpoints.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    let max_nodes = MAX_VISIBLE_ENDPOINTS;
    let latency_config = &app.latency_config;
    let hidden_endpoint_count = sorted_endpoints.len().saturating_sub(max_nodes);
    
    // First pass: classify all endpoints
    let endpoint_data: Vec<_> = sorted_endpoints
        .iter()
        .take(max_nodes)
        .map(|(addr, conns)| {
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

            let label = if addr.len() > 15 {
                format!("{}...", &addr[..12])
            } else {
                (*addr).to_string()
            };

            let latency_bucket = classify_latency(None, latency_config);
            let is_listen_socket = *addr == "0.0.0.0" && conns.iter().all(|c| c.state == ConnectionState::Listen);
            let endpoint_type = classify_endpoint(addr, is_listen_socket);

            (label, state, conns.len(), latency_bucket, endpoint_type)
        })
        .collect();
    
    let all_conn_counts: Vec<usize> = endpoint_data.iter().map(|(_, _, count, _, _)| *count).collect();
    
    // Calculate adaptive layout based on canvas size
    // Larger terminals get larger ring radii for better spacing
    let canvas_width_cells = chunks[1].width.saturating_sub(2) as f64;
    let canvas_height_cells = chunks[1].height.saturating_sub(1) as f64;
    
    // Use the smaller dimension to determine ring scaling
    let smaller_dimension = canvas_width_cells.min(canvas_height_cells);
    
    // Scale ring radii based on canvas size
    // Base radii: [25.0, 35.0, 45.0]
    // Scale factor starts at 1.0 for small terminals (≤30 cells)
    // Scales up more aggressively to utilize large terminal space
    // Max scale factor 3.5 for very large terminals (≥100 cells)
    let scale_factor = ((smaller_dimension - 30.0) / 20.0 + 1.0).clamp(1.0, 3.5);
    
    let layout_config = LayoutConfig {
        ring_low: RING_RADII[0] * scale_factor,
        ring_medium: RING_RADII[1] * scale_factor,
        ring_high: RING_RADII[2] * scale_factor,
        edge_padding: MIN_EDGE_PADDING,
        is_adaptive: scale_factor > 1.0,
    };
    
    // Count endpoints per latency bucket for position calculation
    let mut bucket_counts: HashMap<LatencyBucket, usize> = HashMap::new();
    for (_, _, _, bucket, _) in &endpoint_data {
        *bucket_counts.entry(*bucket).or_insert(0) += 1;
    }
    
    let mut bucket_indices: HashMap<LatencyBucket, usize> = HashMap::new();
    
    // Second pass: calculate positions using index-based distribution
    let nodes: Vec<EndpointNode> = endpoint_data
        .into_iter()
        .map(|(label, state, conn_count, latency_bucket, endpoint_type)| {
            let idx_in_bucket = *bucket_indices.entry(latency_bucket).or_insert(0);
            let total_in_bucket = *bucket_counts.get(&latency_bucket).unwrap_or(&1);
            *bucket_indices.get_mut(&latency_bucket).unwrap() += 1;
            
            let (x, y) = calculate_endpoint_position(idx_in_bucket, total_in_bucket, latency_bucket, &layout_config);
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
    let should_draw_rings = has_latency_data(&nodes);
    let animations_enabled = app.graveyard_settings.animations_enabled;
    let pulse_phase = app.pulse_phase;
    let edge_count = nodes.len();
    let animation_reduced = app.animation_reduced;
    let labels_enabled = app.graveyard_settings.labels_enabled;
    let overdrive_enabled = app.graveyard_settings.overdrive_enabled;
    
    // Calculate canvas dimensions for proper aspect ratio
    // Braille markers: each cell is 2x4 dots, so we multiply accordingly
    let canvas_width_cells = chunks[1].width.saturating_sub(2) as f64; // subtract border
    let canvas_height_cells = chunks[1].height.saturating_sub(1) as f64; // subtract border
    
    // Terminal cells are typically ~2:1 aspect ratio (taller than wide)
    // Braille: 2 dots wide, 4 dots tall per cell
    // So actual pixel ratio is: width_cells * 2 : height_cells * 4 = width_cells : height_cells * 2
    let canvas_pixel_width = canvas_width_cells * 2.0;
    let canvas_pixel_height = canvas_height_cells * 4.0;
    
    // Calculate x_bounds to maintain square coordinate space centered on screen
    // y_bounds stays [0, 100], x_bounds scales based on aspect ratio
    let aspect_ratio = canvas_pixel_width / canvas_pixel_height.max(1.0);
    let x_range = 100.0 * aspect_ratio;
    let x_center = x_range / 2.0;
    
    // Transform node x coordinates from 0-100 space to aspect-ratio adjusted space
    // Nodes are calculated around HOST_CENTER (50, 50), so we need to:
    // 1. Translate to origin (x - 50)
    // 2. Scale by aspect ratio (keep y unchanged, scale x)
    // 3. Translate to new center (+ x_center)
    let nodes: Vec<EndpointNode> = nodes
        .into_iter()
        .map(|mut node| {
            // Transform x coordinate: (x - 50) + x_center
            // This shifts the node positions to be centered in the wider canvas
            node.x = (node.x - 50.0) + x_center;
            node
        })
        .collect();
    
    // For closure capture
    let canvas_height = canvas_pixel_height;

    // Canvas with Braille markers
    let canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(NEON_PURPLE)),
        )
        .marker(Marker::Braille)
        .x_bounds([0.0, x_range])
        .y_bounds([0.0, 100.0])
        .paint(move |ctx| {
            // Center point adjusted for aspect ratio
            let cx = x_center;
            let cy = 50.0;
            
            // Draw latency rings first (behind everything else)
            // Uses adaptive layout config for ring radii
            if should_draw_rings {
                draw_latency_rings(ctx, &layout_config, |ctx, x, y, style| {
                    ctx.print(x, y, Span::styled("·", style));
                });
            }

            // Calculate coffin exclusion zone radius based on selected variant
            // This ensures connection lines don't overlap the coffin silhouette
            let coffin_variant = get_coffin_variant_for_canvas(canvas_height, &center_label);
            let coffin_radius = coffin_exclusion_radius(coffin_variant);
            
            for node in &nodes {
                let line_color = match node.state {
                    ConnectionState::Established => TOXIC_GREEN,
                    ConnectionState::TimeWait | ConnectionState::CloseWait => PUMPKIN_ORANGE,
                    ConnectionState::SynSent | ConnectionState::SynRecv => Color::Yellow,
                    ConnectionState::Close => BLOOD_RED,
                    _ => pulse_color,
                };

                let dx = node.x - cx;
                let dy = node.y - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                
                let (start_x, start_y) = if dist > coffin_radius {
                    let ratio = coffin_radius / dist;
                    (cx + dx * ratio, cy + dy * ratio)
                } else {
                    (cx, cy)
                };

                // Draw base edge line
                ctx.draw(&CanvasLine {
                    x1: start_x,
                    y1: start_y,
                    x2: node.x,
                    y2: node.y,
                    color: line_color,
                });
                
                // Draw particles if animations enabled
                if animations_enabled {
                    let is_visible = node.x >= 0.0 && node.x <= 100.0 
                                  && node.y >= 0.0 && node.y <= 100.0;
                    
                    if !is_visible {
                        continue;
                    }
                    
                    let particle_color = match node.state {
                        ConnectionState::TimeWait | ConnectionState::CloseWait => PUMPKIN_ORANGE,
                        ConnectionState::Established => {
                            if node.latency_bucket == LatencyBucket::High {
                                PUMPKIN_ORANGE
                            } else {
                                TOXIC_GREEN
                            }
                        }
                        _ => NEON_PURPLE,
                    };
                    
                    let particle_offsets: &[f32] = if animation_reduced || edge_count > PARTICLE_REDUCTION_THRESHOLD {
                        &REDUCED_PARTICLE_OFFSETS
                    } else {
                        &PARTICLE_OFFSETS
                    };
                    
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

            // Draw coffin block at center
            draw_coffin_block(ctx, &center_label, overdrive_enabled, canvas_height, cx, cy);

            // Draw endpoint nodes
            for node in &nodes {
                let icon = if overdrive_enabled {
                    let overdrive_icon = get_overdrive_icon(node.state, node.latency_bucket);
                    if node.is_heavy_talker {
                        format!("{}👑", overdrive_icon)
                    } else {
                        overdrive_icon.to_string()
                    }
                } else {
                    node.endpoint_type.icon_with_badge(node.is_heavy_talker)
                };
                
                let color = match node.state {
                    ConnectionState::TimeWait | ConnectionState::CloseWait => PUMPKIN_ORANGE,
                    ConnectionState::Close => BLOOD_RED,
                    _ => node.endpoint_type.color(),
                };

                ctx.print(node.x, node.y, Span::styled(icon, Style::default().fg(color)));

                if labels_enabled {
                    let label = format!("{} ({})", node.label, node.conn_count);
                    ctx.print(
                        node.x - 6.0,
                        node.y - 4.0,
                        Span::styled(label, Style::default().fg(color)),
                    );
                }
            }

            // Show message if no connections
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

            // Show "... and N more" indicator
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::LatencyConfig;

    // ============================================================================
    // Test endpoint classification
    // Requirements: 3.1, 3.2, 3.5
    // ============================================================================

    #[test]
    fn test_classify_endpoint_localhost() {
        assert_eq!(classify_endpoint("127.0.0.1", false), EndpointType::Localhost);
        assert_eq!(classify_endpoint("::1", false), EndpointType::Localhost);
        assert_eq!(classify_endpoint("0.0.0.0", false), EndpointType::Localhost);
    }

    #[test]
    fn test_classify_endpoint_rfc1918_class_a() {
        assert_eq!(classify_endpoint("10.0.0.1", false), EndpointType::Private);
        assert_eq!(classify_endpoint("10.255.255.255", false), EndpointType::Private);
        assert_eq!(classify_endpoint("10.100.50.25", false), EndpointType::Private);
    }

    #[test]
    fn test_classify_endpoint_rfc1918_class_b() {
        assert_eq!(classify_endpoint("172.16.0.1", false), EndpointType::Private);
        assert_eq!(classify_endpoint("172.31.255.255", false), EndpointType::Private);
        assert_eq!(classify_endpoint("172.20.100.50", false), EndpointType::Private);
        assert_eq!(classify_endpoint("172.15.0.1", false), EndpointType::Public);
        assert_eq!(classify_endpoint("172.32.0.1", false), EndpointType::Public);
    }

    #[test]
    fn test_classify_endpoint_rfc1918_class_c() {
        assert_eq!(classify_endpoint("192.168.0.1", false), EndpointType::Private);
        assert_eq!(classify_endpoint("192.168.255.255", false), EndpointType::Private);
        assert_eq!(classify_endpoint("192.168.1.100", false), EndpointType::Private);
        assert_eq!(classify_endpoint("192.169.0.1", false), EndpointType::Public);
        assert_eq!(classify_endpoint("192.167.0.1", false), EndpointType::Public);
    }

    #[test]
    fn test_classify_endpoint_public() {
        assert_eq!(classify_endpoint("8.8.8.8", false), EndpointType::Public);
        assert_eq!(classify_endpoint("1.1.1.1", false), EndpointType::Public);
        assert_eq!(classify_endpoint("203.0.113.50", false), EndpointType::Public);
        assert_eq!(classify_endpoint("198.51.100.1", false), EndpointType::Public);
    }

    #[test]
    fn test_classify_endpoint_listen_only() {
        assert_eq!(classify_endpoint("0.0.0.0", true), EndpointType::ListenOnly);
        assert_eq!(classify_endpoint("127.0.0.1", true), EndpointType::ListenOnly);
        assert_eq!(classify_endpoint("192.168.1.1", true), EndpointType::ListenOnly);
    }

    #[test]
    fn test_endpoint_type_icons() {
        assert_eq!(EndpointType::Localhost.icon(), "⚰️");
        assert_eq!(EndpointType::Private.icon(), "🪦");
        assert_eq!(EndpointType::Public.icon(), "🎃");
        assert_eq!(EndpointType::ListenOnly.icon(), "🕯");
    }

    #[test]
    fn test_endpoint_type_colors() {
        assert_eq!(EndpointType::Localhost.color(), TOXIC_GREEN);
        assert_eq!(EndpointType::Private.color(), BONE_WHITE);
        assert_eq!(EndpointType::Public.color(), PUMPKIN_ORANGE);
        assert_eq!(EndpointType::ListenOnly.color(), NEON_PURPLE);
    }

    #[test]
    fn test_endpoint_type_icon_with_badge() {
        assert_eq!(EndpointType::Public.icon_with_badge(false), "🎃");
        assert_eq!(EndpointType::Public.icon_with_badge(true), "🎃👑");
        assert_eq!(EndpointType::Private.icon_with_badge(true), "🪦👑");
    }

    // ============================================================================
    // Test latency bucket classification
    // Requirements: 1.2, 1.3, 1.4, 1.5
    // ============================================================================

    #[test]
    fn test_classify_latency_low() {
        let config = LatencyConfig::default();
        assert_eq!(classify_latency(Some(0), &config), LatencyBucket::Low);
        assert_eq!(classify_latency(Some(25), &config), LatencyBucket::Low);
        assert_eq!(classify_latency(Some(49), &config), LatencyBucket::Low);
    }

    #[test]
    fn test_classify_latency_medium() {
        let config = LatencyConfig::default();
        assert_eq!(classify_latency(Some(50), &config), LatencyBucket::Medium);
        assert_eq!(classify_latency(Some(100), &config), LatencyBucket::Medium);
        assert_eq!(classify_latency(Some(200), &config), LatencyBucket::Medium);
    }

    #[test]
    fn test_classify_latency_high() {
        let config = LatencyConfig::default();
        assert_eq!(classify_latency(Some(201), &config), LatencyBucket::High);
        assert_eq!(classify_latency(Some(500), &config), LatencyBucket::High);
        assert_eq!(classify_latency(Some(1000), &config), LatencyBucket::High);
    }

    #[test]
    fn test_classify_latency_unknown() {
        let config = LatencyConfig::default();
        assert_eq!(classify_latency(None, &config), LatencyBucket::Unknown);
    }

    #[test]
    fn test_classify_latency_custom_thresholds() {
        let config = LatencyConfig {
            low_threshold_ms: 100,
            high_threshold_ms: 500,
        };
        
        assert_eq!(classify_latency(Some(50), &config), LatencyBucket::Low);
        assert_eq!(classify_latency(Some(99), &config), LatencyBucket::Low);
        assert_eq!(classify_latency(Some(100), &config), LatencyBucket::Medium);
        assert_eq!(classify_latency(Some(300), &config), LatencyBucket::Medium);
        assert_eq!(classify_latency(Some(500), &config), LatencyBucket::Medium);
        assert_eq!(classify_latency(Some(501), &config), LatencyBucket::High);
    }

    // ============================================================================
    // Test heavy talker detection
    // Requirements: 3.4
    // ============================================================================

    #[test]
    fn test_is_heavy_talker_top_5() {
        let all_counts = vec![100, 80, 60, 40, 20, 10, 5];
        
        assert!(is_heavy_talker(100, &all_counts));
        assert!(is_heavy_talker(80, &all_counts));
        assert!(is_heavy_talker(60, &all_counts));
        assert!(is_heavy_talker(40, &all_counts));
        assert!(is_heavy_talker(20, &all_counts));
        assert!(!is_heavy_talker(10, &all_counts));
        assert!(!is_heavy_talker(5, &all_counts));
    }

    #[test]
    fn test_is_heavy_talker_fewer_than_5() {
        let all_counts = vec![50, 30, 10];
        
        assert!(is_heavy_talker(50, &all_counts));
        assert!(is_heavy_talker(30, &all_counts));
        assert!(is_heavy_talker(10, &all_counts));
    }

    #[test]
    fn test_is_heavy_talker_empty() {
        let all_counts: Vec<usize> = vec![];
        assert!(!is_heavy_talker(10, &all_counts));
    }

    #[test]
    fn test_is_heavy_talker_zero_count() {
        let all_counts = vec![10, 5, 0, 0, 0];
        
        assert!(!is_heavy_talker(0, &all_counts));
        assert!(is_heavy_talker(10, &all_counts));
        assert!(is_heavy_talker(5, &all_counts));
    }

    #[test]
    fn test_is_heavy_talker_ties() {
        let all_counts = vec![100, 50, 50, 50, 50, 10];
        
        assert!(is_heavy_talker(100, &all_counts));
        assert!(is_heavy_talker(50, &all_counts));
        assert!(!is_heavy_talker(10, &all_counts));
    }

    // ============================================================================
    // Test particle position calculation
    // Requirements: 2.2
    // ============================================================================

    #[test]
    fn test_particle_position_at_start() {
        let start = (50.0, 50.0);
        let end = (80.0, 30.0);
        
        let pos = particle_position(start, end, 0.0, 0.0);
        assert!((pos.0 - 50.0).abs() < 0.001);
        assert!((pos.1 - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_particle_position_at_middle() {
        let start = (50.0, 50.0);
        let end = (80.0, 30.0);
        
        let pos = particle_position(start, end, 0.5, 0.0);
        assert!((pos.0 - 65.0).abs() < 0.001);
        assert!((pos.1 - 40.0).abs() < 0.001);
    }

    #[test]
    fn test_particle_position_at_end() {
        let start = (50.0, 50.0);
        let end = (80.0, 30.0);
        
        let pos = particle_position(start, end, 1.0, 0.0);
        assert!((pos.0 - 50.0).abs() < 0.001);
        assert!((pos.1 - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_particle_position_with_offset() {
        let start = (0.0, 0.0);
        let end = (100.0, 100.0);
        
        let pos = particle_position(start, end, 0.0, 0.33);
        assert!((pos.0 - 33.0).abs() < 0.001);
        assert!((pos.1 - 33.0).abs() < 0.001);
        
        let pos = particle_position(start, end, 0.0, 0.66);
        assert!((pos.0 - 66.0).abs() < 0.001);
        assert!((pos.1 - 66.0).abs() < 0.001);
    }

    #[test]
    fn test_particle_position_wrapping() {
        let start = (0.0, 0.0);
        let end = (100.0, 0.0);
        
        let pos = particle_position(start, end, 0.8, 0.33);
        let expected_t = (0.8 + 0.33) % 1.0;
        assert!((pos.0 - expected_t * 100.0).abs() < 0.001);
    }

    // ============================================================================
    // Test endpoint position calculation
    // Requirements: 1.2, 2.1, 2.3
    // ============================================================================

    #[test]
    fn test_calculate_endpoint_position_ring_selection() {
        // Use default layout config (fixed radii)
        let layout = LayoutConfig::default();
        
        let (x_low, y_low) = calculate_endpoint_position(0, 1, LatencyBucket::Low, &layout);
        let (x_med, y_med) = calculate_endpoint_position(0, 1, LatencyBucket::Medium, &layout);
        let (x_high, y_high) = calculate_endpoint_position(0, 1, LatencyBucket::High, &layout);
        
        let dist_low = ((x_low - 50.0).powi(2) + (y_low - 50.0).powi(2)).sqrt();
        let dist_med = ((x_med - 50.0).powi(2) + (y_med - 50.0).powi(2)).sqrt();
        let dist_high = ((x_high - 50.0).powi(2) + (y_high - 50.0).powi(2)).sqrt();
        
        assert!(dist_low < dist_med);
        assert!(dist_med < dist_high);
    }

    #[test]
    fn test_calculate_endpoint_position_unknown_fallback() {
        // Use default layout config (fixed radii)
        let layout = LayoutConfig::default();
        
        let (x_unknown, y_unknown) = calculate_endpoint_position(0, 1, LatencyBucket::Unknown, &layout);
        let (x_medium, y_medium) = calculate_endpoint_position(0, 1, LatencyBucket::Medium, &layout);
        
        let dist_unknown = ((x_unknown - 50.0).powi(2) + (y_unknown - 50.0).powi(2)).sqrt();
        let dist_medium = ((x_medium - 50.0).powi(2) + (y_medium - 50.0).powi(2)).sqrt();
        
        assert!((dist_unknown - dist_medium).abs() < 5.0);
    }

    #[test]
    fn test_calculate_endpoint_position_bounds() {
        // Use default layout config (fixed radii)
        let layout = LayoutConfig::default();
        
        for i in 0..10 {
            for bucket in [LatencyBucket::Low, LatencyBucket::Medium, LatencyBucket::High] {
                let (x, y) = calculate_endpoint_position(i, 10, bucket, &layout);
                assert!(x >= layout.edge_padding && x <= 100.0 - layout.edge_padding, 
                    "x={} out of bounds for padding={}", x, layout.edge_padding);
                assert!(y >= layout.edge_padding && y <= 100.0 - layout.edge_padding, 
                    "y={} out of bounds for padding={}", y, layout.edge_padding);
            }
        }
    }
    
    #[test]
    fn test_calculate_endpoint_position_ring_ordering() {
        // Test that ring ordering is preserved (Low < Medium < High)
        let layout = LayoutConfig::default();
        
        let (x_low, y_low) = calculate_endpoint_position(0, 1, LatencyBucket::Low, &layout);
        let (x_high, y_high) = calculate_endpoint_position(0, 1, LatencyBucket::High, &layout);
        
        let dist_low = ((x_low - 50.0).powi(2) + (y_low - 50.0).powi(2)).sqrt();
        let dist_high = ((x_high - 50.0).powi(2) + (y_high - 50.0).powi(2)).sqrt();
        
        // Verify ring ordering is preserved
        assert!(dist_low < dist_high, "Low ring should be closer than high ring");
    }

    #[test]
    fn test_has_latency_data() {
        let nodes_with_data = vec![
            EndpointNode {
                label: "test".to_string(),
                x: 50.0,
                y: 50.0,
                state: ConnectionState::Established,
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
                state: ConnectionState::Established,
                conn_count: 1,
                latency_bucket: LatencyBucket::Unknown,
                endpoint_type: EndpointType::Public,
                is_heavy_talker: false,
            },
        ];
        assert!(!has_latency_data(&nodes_without_data));
        
        let empty_nodes: Vec<EndpointNode> = vec![];
        assert!(!has_latency_data(&empty_nodes));
    }

    // ============================================================================
    // Test Classic Coffin Rendering System - HARDCODED TEMPLATES
    // Requirements: 3.1
    // These tests ensure the coffin ASCII art remains STABLE and UNCHANGED
    // ============================================================================

    // ============================================================================
    // COFFIN TEMPLATE STABILITY TESTS
    // These tests ensure the coffin ASCII art is NEVER accidentally changed.
    // If these tests fail, it means someone modified the hardcoded templates.
    // ============================================================================

    #[test]
    fn large_coffin_shape_is_stable() {
        // This test ensures the Large coffin template is NEVER changed
        // DO NOT MODIFY THESE ASSERTIONS - they are the source of truth
        let coffin = build_large_coffin("HOST");
        
        assert_eq!(coffin.variant, CoffinVariant::Large);
        assert_eq!(coffin.height, 4, "Large coffin must be exactly 4 lines");
        assert_eq!(coffin.width, LARGE_COFFIN_WIDTH, "Large coffin width must match constant");
        assert_eq!(coffin.lines.len(), 4);
        
        // Verify all lines are exactly 14 chars
        for (i, line) in coffin.lines.iter().enumerate() {
            assert_eq!(line.chars().count(), 14, "Line {} must be exactly 14 chars", i);
        }
        
        // Verify EXACT template structure (14 chars wide)
        // Line 0: Top point
        assert_eq!(coffin.lines[0], "   /‾‾‾‾‾‾\\   ", "Line 0 (top) must match exactly");
        // Line 1: HOST placeholder (centered in 6-char space)
        assert!(coffin.lines[1].contains("HOST"), "Line 1 must contain HOST");
        assert!(coffin.lines[1].starts_with("  /"), "Line 1 must start with '  /'");
        assert!(coffin.lines[1].ends_with("\\  "), "Line 1 must end with '\\  '");
        // Line 2: Lower body
        assert_eq!(coffin.lines[2], "  \\        /  ", "Line 2 must match exactly");
        // Line 3: Bottom base
        assert_eq!(coffin.lines[3], "   \\______/   ", "Line 3 (bottom) must match exactly");
    }

    #[test]
    fn mid_coffin_shape_is_stable() {
        // This test ensures the Mid coffin template is NEVER changed
        // DO NOT MODIFY THESE ASSERTIONS - they are the source of truth
        let coffin = build_mid_coffin("HOST");
        
        assert_eq!(coffin.variant, CoffinVariant::Mid);
        assert_eq!(coffin.height, 3, "Mid coffin must be exactly 3 lines");
        assert_eq!(coffin.width, MID_COFFIN_WIDTH, "Mid coffin width must match constant");
        assert_eq!(coffin.lines.len(), 3);
        
        // Verify all lines are exactly 11 chars
        for (i, line) in coffin.lines.iter().enumerate() {
            assert_eq!(line.chars().count(), 11, "Line {} must be exactly 11 chars", i);
        }
        
        // Verify EXACT template structure (11 chars wide)
        // Line 0: Top
        assert_eq!(coffin.lines[0], " /‾‾‾‾‾‾\\  ", "Line 0 (top) must match exactly");
        // Line 1: HOST placeholder
        assert!(coffin.lines[1].contains("HOST"), "Line 1 must contain HOST");
        assert!(coffin.lines[1].starts_with("/"), "Line 1 must start with '/'");
        assert!(coffin.lines[1].ends_with(" "), "Line 1 must end with space");
        // Line 2: Bottom base
        assert_eq!(coffin.lines[2], " \\______/  ", "Line 2 (bottom) must match exactly");
    }

    #[test]
    fn label_coffin_format_is_stable() {
        // This test ensures the Label coffin format is NEVER changed
        // Format: [⚰ HOST]
        let coffin = build_label_coffin("HOST", 20);
        
        assert_eq!(coffin.variant, CoffinVariant::Label);
        assert_eq!(coffin.height, 1, "Label coffin must be exactly 1 line");
        assert_eq!(coffin.lines.len(), 1);
        
        // Verify EXACT format
        assert_eq!(coffin.lines[0], "[⚰ HOST]", "Label format must be [⚰ HOST]");
    }

    #[test]
    fn test_coffin_name_truncation() {
        // Long name should be truncated with ".."
        let coffin = build_large_coffin("kafka-broker-1");
        
        // Name should be truncated to fit LARGE_COFFIN_MAX_NAME (6 chars)
        let has_truncated = coffin.lines.iter().any(|line| line.contains(".."));
        assert!(has_truncated, "Long name should be truncated with ..");
        
        // Verify the truncated name fits within the coffin
        let host_line = &coffin.lines[1];
        assert_eq!(host_line.chars().count(), LARGE_COFFIN_WIDTH, 
            "Truncated name line must maintain coffin width");
    }

    #[test]
    fn test_coffin_graceful_degradation() {
        // Test degradation from Large to Mid to Label
        // Canvas-to-char conversion: char_height = area_height / 4.0, char_width = area_width / 1.0
        
        // Large coffin at large canvas (100x100 -> 100 chars wide, 25 chars tall)
        // Requires: width >= 14, height >= 5
        let large = choose_coffin_variant(100.0, 100.0, "TEST");
        assert_eq!(large.variant, CoffinVariant::Large, 
            "Large canvas should use Large coffin");
        
        // Mid coffin at medium canvas (13x16 -> 13 chars wide, 4 chars tall)
        // width < 14 but >= 11, height >= 3 -> Mid
        let mid = choose_coffin_variant(13.0, 16.0, "TEST");
        assert_eq!(mid.variant, CoffinVariant::Mid,
            "Medium canvas should use Mid coffin");
        
        // Label only at small canvas (10x4 -> 10 chars wide, 1 char tall)
        // width < 11 or height < 3 forces Label
        let label = choose_coffin_variant(10.0, 4.0, "TEST");
        assert_eq!(label.variant, CoffinVariant::Label,
            "Small canvas should use Label coffin");
    }

    #[test]
    fn test_coffin_dimensions_are_fixed() {
        // Verify coffin dimensions match constants
        // These dimensions are FIXED and should never change
        let large = build_large_coffin("X");
        assert_eq!(large.width, LARGE_COFFIN_WIDTH);
        assert_eq!(large.height, LARGE_COFFIN_HEIGHT);
        assert_eq!(large.width, 14, "Large coffin width constant must be 14");
        assert_eq!(large.height, 4, "Large coffin height constant must be 4");
        
        let mid = build_mid_coffin("X");
        assert_eq!(mid.width, MID_COFFIN_WIDTH);
        assert_eq!(mid.height, MID_COFFIN_HEIGHT);
        assert_eq!(mid.width, 11, "Mid coffin width constant must be 11");
        assert_eq!(mid.height, 3, "Mid coffin height constant must be 3");
    }

    #[test]
    fn test_coffin_exclusion_radius() {
        // Test that exclusion radii are appropriate for each variant
        let large_radius = coffin_exclusion_radius(CoffinVariant::Large);
        let mid_radius = coffin_exclusion_radius(CoffinVariant::Mid);
        let label_radius = coffin_exclusion_radius(CoffinVariant::Label);
        
        // Larger coffins need larger exclusion zones
        assert!(large_radius > mid_radius, "Large coffin needs larger exclusion");
        assert!(mid_radius > label_radius, "Mid coffin needs larger exclusion than Label");
        
        // Verify specific values
        assert_eq!(large_radius, 15.0, "Large coffin exclusion radius");
        assert_eq!(mid_radius, 12.0, "Mid coffin exclusion radius");
        assert_eq!(label_radius, 8.0, "Label coffin exclusion radius");
    }

    #[test]
    fn test_truncate_host_name() {
        // Test truncation helper function
        assert_eq!(truncate_host_name("HOST", 10), "HOST");
        assert_eq!(truncate_host_name("kafka-broker-1", 6), "kafk..");
        assert_eq!(truncate_host_name("AB", 2), "AB");
        assert_eq!(truncate_host_name("ABCD", 3), "ABC"); // Too short for ".."
        assert_eq!(truncate_host_name("ABCDEF", 4), "AB..");
    }

    #[test]
    fn test_center_pad() {
        // Test center padding helper function
        assert_eq!(center_pad("X", 5), "  X  ");
        assert_eq!(center_pad("AB", 6), "  AB  ");
        assert_eq!(center_pad("HOST", 6), " HOST ");
        assert_eq!(center_pad("TOOLONG", 4), "TOOLONG"); // No truncation, just return as-is
        assert_eq!(center_pad("ABC", 4), "ABC "); // Odd padding goes to right
    }

    #[test]
    fn test_coffin_with_various_hostnames() {
        // Test coffin rendering with various hostname lengths
        
        // Short name (HOST is now on line[1])
        let short = build_large_coffin("DB");
        assert!(short.lines[1].contains("DB"), "Short name should be visible");
        
        // Exact fit name (6 chars)
        let exact = build_large_coffin("KAFKA1");
        assert!(exact.lines[1].contains("KAFKA1"), "Exact fit name should be visible");
        
        // Long name (should truncate)
        let long = build_large_coffin("very-long-hostname");
        assert!(long.lines[1].contains(".."), "Long name should be truncated");
        assert!(!long.lines[1].contains("very-long"), "Full long name should not appear");
    }

    #[test]
    fn test_label_coffin_width_constraint() {
        // Test that label coffin respects max_width
        let narrow = build_label_coffin("kafka-broker-1", 10);
        assert!(narrow.width <= 10, "Label should respect max_width");
        
        let wide = build_label_coffin("kafka-broker-1", 30);
        // With 30 chars available, should show more of the name
        assert!(wide.lines[0].len() > narrow.lines[0].len(), 
            "Wider constraint should show more of the name");
    }
}
