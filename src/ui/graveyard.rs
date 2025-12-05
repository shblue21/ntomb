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

/// Minimum canvas height (in canvas units) to use the large coffin design
/// Below this threshold, the mini coffin (single line) is used
const LARGE_COFFIN_MIN_HEIGHT: f64 = 50.0;


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

/// Draw the coffin block on the canvas at the HOST center
/// 
/// Renders a hexagonal coffin shape for the central HOST node.
/// Automatically switches to mini (single-line) mode when canvas is small.
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
fn draw_large_coffin(
    ctx: &mut ratatui::widgets::canvas::Context<'_>,
    cx: f64,
    cy: f64,
    display_name: &str,
    coffin_color: Color,
) {
    // Calculate widths based on name length
    let content_width = 6 + display_name.len();
    let top_bar_width = content_width;
    let bottom_bar_width = content_width;
    
    // Build coffin lines using ASCII-compatible characters
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
fn draw_mini_coffin(
    ctx: &mut ratatui::widgets::canvas::Context<'_>,
    cx: f64,
    cy: f64,
    display_name: &str,
    coffin_color: Color,
) {
    let content = format!("⚰ {}", display_name);
    
    // Calculate centering
    let display_width = 2 + 1 + display_name.len();
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
/// Requirements: 1.1, 1.6
pub fn draw_latency_rings<F>(ctx: &mut ratatui::widgets::canvas::Context<'_>, draw_point: F)
where
    F: Fn(&mut ratatui::widgets::canvas::Context<'_>, f64, f64, Style),
{
    let (cx, cy) = HOST_CENTER;
    
    for (ring_idx, radius) in RING_RADII.iter().enumerate() {
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
/// Positions endpoints on concentric rings around HOST_CENTER based on their latency.
/// 
/// Requirements: 1.2, 1.3, 1.4, 1.5
pub fn calculate_endpoint_position(
    endpoint_idx: usize,
    total_in_bucket: usize,
    latency_bucket: LatencyBucket,
) -> (f64, f64) {
    let (cx, cy) = HOST_CENTER;
    
    // Select ring radius based on latency bucket
    let radius = match latency_bucket {
        LatencyBucket::Low => RING_RADII[0],
        LatencyBucket::Medium => RING_RADII[1],
        LatencyBucket::High => RING_RADII[2],
        LatencyBucket::Unknown => RING_RADII[1],
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
    
    // Clamp to canvas bounds with padding
    (x.clamp(5.0, 95.0), y.clamp(5.0, 95.0))
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
    
    // Count endpoints per latency bucket
    let mut bucket_counts: HashMap<LatencyBucket, usize> = HashMap::new();
    for (_, _, _, bucket, _) in &endpoint_data {
        *bucket_counts.entry(*bucket).or_insert(0) += 1;
    }
    
    let mut bucket_indices: HashMap<LatencyBucket, usize> = HashMap::new();
    
    // Second pass: calculate positions
    let nodes: Vec<EndpointNode> = endpoint_data
        .into_iter()
        .map(|(label, state, conn_count, latency_bucket, endpoint_type)| {
            let idx_in_bucket = *bucket_indices.entry(latency_bucket).or_insert(0);
            let total_in_bucket = *bucket_counts.get(&latency_bucket).unwrap_or(&1);
            *bucket_indices.get_mut(&latency_bucket).unwrap() += 1;
            
            let (x, y) = calculate_endpoint_position(idx_in_bucket, total_in_bucket, latency_bucket);
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
            
            // Draw latency rings first (behind everything else)
            if should_draw_rings {
                draw_latency_rings(ctx, |ctx, x, y, style| {
                    ctx.print(x, y, Span::styled("·", style));
                });
            }

            // Coffin exclusion zone radius
            let coffin_radius = 8.0;
            
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
            draw_coffin_block(ctx, &center_label, overdrive_enabled, canvas_height);

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
    // Requirements: 1.2, 1.3, 1.4, 1.5
    // ============================================================================

    #[test]
    fn test_calculate_endpoint_position_ring_selection() {
        let (x_low, y_low) = calculate_endpoint_position(0, 1, LatencyBucket::Low);
        let (x_med, y_med) = calculate_endpoint_position(0, 1, LatencyBucket::Medium);
        let (x_high, y_high) = calculate_endpoint_position(0, 1, LatencyBucket::High);
        
        let dist_low = ((x_low - 50.0).powi(2) + (y_low - 50.0).powi(2)).sqrt();
        let dist_med = ((x_med - 50.0).powi(2) + (y_med - 50.0).powi(2)).sqrt();
        let dist_high = ((x_high - 50.0).powi(2) + (y_high - 50.0).powi(2)).sqrt();
        
        assert!(dist_low < dist_med);
        assert!(dist_med < dist_high);
    }

    #[test]
    fn test_calculate_endpoint_position_unknown_fallback() {
        let (x_unknown, y_unknown) = calculate_endpoint_position(0, 1, LatencyBucket::Unknown);
        let (x_medium, y_medium) = calculate_endpoint_position(0, 1, LatencyBucket::Medium);
        
        let dist_unknown = ((x_unknown - 50.0).powi(2) + (y_unknown - 50.0).powi(2)).sqrt();
        let dist_medium = ((x_medium - 50.0).powi(2) + (y_medium - 50.0).powi(2)).sqrt();
        
        assert!((dist_unknown - dist_medium).abs() < 5.0);
    }

    #[test]
    fn test_calculate_endpoint_position_bounds() {
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
}
