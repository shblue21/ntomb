// Soul Inspector rendering module
//
// Renders the detail panel showing selected process/connection information,
// traffic sparkline, and socket list.
//
// The Soul Inspector displays real-time data about the currently selected
// target (process or connection) from AppState.

use crate::app::{AppState, GraveyardMode};
use crate::net::{Connection, ConnectionState};
use crate::theme::{get_refresh_color, get_status_text, BLOOD_RED, BONE_WHITE, NEON_PURPLE, PUMPKIN_ORANGE, TOXIC_GREEN};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Sparkline},
    Frame,
};

// ============================================================================
// Soul Inspector View Model
// ============================================================================

/// View model for Soul Inspector panel
/// 
/// Contains all data needed to render the Soul Inspector, extracted from AppState.
/// This separates data extraction from rendering logic.
#[derive(Debug, Clone)]
pub struct SoulInspectorView {
    /// Target name to display (process name, connection endpoint, or "HOST")
    pub target_name: String,
    /// Target icon (⚰️ for process, 🔗 for connection, 🏠 for host)
    pub target_icon: String,
    /// Process ID if available
    pub pid: Option<i32>,
    /// Parent process ID (not available in current data model, reserved for future use)
    #[allow(dead_code)]
    pub ppid: Option<i32>,
    /// User name (not available in current data model, reserved for future use)
    #[allow(dead_code)]
    pub user: Option<String>,
    /// State icon (🟢, 🟡, 🔴)
    pub state_icon: String,
    /// State text (e.g., "ESTABLISHED (Alive)")
    pub state_text: String,
    /// State color for styling
    pub state_color: Color,
    /// Current UI refresh interval in milliseconds
    pub refresh_ms: u64,
    /// Number of connections for this target
    pub conn_count: usize,
    /// List of connections/sockets for this target
    pub sockets: Vec<SocketInfo>,
    /// Whether this target has suspicious activity
    pub suspicious: bool,
    /// Tags for this target
    pub tags: Vec<String>,
    /// Whether a target is selected
    pub has_selection: bool,
}

/// Socket/connection info for display in the socket list
#[derive(Debug, Clone)]
pub struct SocketInfo {
    /// Display string (e.g., "tcp://127.0.0.1:8080")
    pub display: String,
    /// Remote endpoint if applicable
    pub remote: Option<String>,
    /// Connection state
    pub state: ConnectionState,
}

impl Default for SoulInspectorView {
    fn default() -> Self {
        Self {
            target_name: "No target selected".to_string(),
            target_icon: "👻".to_string(),
            pid: None,
            ppid: None,
            user: None,
            state_icon: "⚪".to_string(),
            state_text: "Idle".to_string(),
            state_color: BONE_WHITE,
            refresh_ms: 500,
            conn_count: 0,
            sockets: Vec::new(),
            suspicious: false,
            tags: Vec::new(),
            has_selection: false,
        }
    }
}

/// Build SoulInspectorView from AppState
/// 
/// Extracts relevant data based on current selection mode:
/// - Host mode: Shows overall host statistics
/// - Process mode: Shows selected process details and its connections
/// - Connection selected: Shows selected connection details
pub fn build_soul_inspector_view(app: &AppState) -> SoulInspectorView {
    let mut view = SoulInspectorView {
        refresh_ms: app.refresh_config.refresh_ms,
        ..Default::default()
    };

    match app.graveyard_mode {
        GraveyardMode::Host => {
            // Host mode - show overall statistics or selected connection
            if let Some(conn_idx) = app.selected_connection {
                // A connection is selected - show its details
                if let Some(conn) = app.connections.get(conn_idx) {
                    build_connection_view(&mut view, conn, &app.connections);
                }
            } else {
                // No selection - show host overview
                build_host_view(&mut view, &app.connections);
            }
        }
        GraveyardMode::Process => {
            // Process mode - show selected process details
            if let Some(pid) = app.selected_process_pid {
                build_process_view(&mut view, pid, &app.connections);
            } else {
                // Process mode but no PID (shouldn't happen normally)
                view.target_name = "No process selected".to_string();
                view.target_icon = "❓".to_string();
            }
        }
    }

    view
}

/// Build view for Host mode (no specific selection)
fn build_host_view(view: &mut SoulInspectorView, connections: &[Connection]) {
    view.target_name = "HOST".to_string();
    view.target_icon = "🏠".to_string();
    view.has_selection = true;
    
    // Count connection states
    let established = connections.iter().filter(|c| c.state == ConnectionState::Established).count();
    let listening = connections.iter().filter(|c| c.state == ConnectionState::Listen).count();
    let other = connections.len() - established - listening;
    
    view.conn_count = connections.len();
    
    // Determine overall state based on connection health
    if connections.is_empty() {
        view.state_icon = "⚪".to_string();
        view.state_text = "No connections".to_string();
        view.state_color = BONE_WHITE;
    } else if established > 0 {
        view.state_icon = "🟢".to_string();
        view.state_text = format!("{} active, {} listening", established, listening);
        view.state_color = TOXIC_GREEN;
    } else if listening > 0 {
        view.state_icon = "🟡".to_string();
        view.state_text = format!("{} listening", listening);
        view.state_color = PUMPKIN_ORANGE;
    } else {
        view.state_icon = "🟠".to_string();
        view.state_text = format!("{} other states", other);
        view.state_color = PUMPKIN_ORANGE;
    }
    
    // Build socket list (show first few connections)
    view.sockets = connections.iter()
        .take(5)
        .map(|c| connection_to_socket_info(c))
        .collect();
    
    // Add tags
    if listening > 0 {
        view.tags.push(format!("server ({})", listening));
    }
    if established > 0 {
        view.tags.push(format!("client ({})", established));
    }
}

/// Build view for a selected connection
fn build_connection_view(view: &mut SoulInspectorView, conn: &Connection, all_connections: &[Connection]) {
    view.has_selection = true;
    view.target_icon = "🔗".to_string();
    
    // Target name: show remote endpoint or local if LISTEN
    if conn.state == ConnectionState::Listen {
        view.target_name = format!("{}:{}", conn.local_addr, conn.local_port);
    } else {
        view.target_name = format!("{}:{}", conn.remote_addr, conn.remote_port);
    }
    
    // Truncate if too long
    if view.target_name.len() > 20 {
        view.target_name = format!("{}...", &view.target_name[..17]);
    }
    
    // PID and process info
    view.pid = conn.pid;
    
    // State
    let (icon, text, color) = connection_state_display(conn.state);
    view.state_icon = icon;
    view.state_text = text;
    view.state_color = color;
    
    // Count connections to same remote
    if conn.state != ConnectionState::Listen {
        view.conn_count = all_connections.iter()
            .filter(|c| c.remote_addr == conn.remote_addr)
            .count();
    } else {
        view.conn_count = 1;
    }
    
    // Socket info
    view.sockets = vec![connection_to_socket_info(conn)];
    
    // Add process name as tag if available
    if let Some(ref name) = conn.process_name {
        view.tags.push(name.clone());
    }
    
    // Check for suspicious patterns
    check_suspicious_patterns(view, conn);
}

/// Build view for a selected process
fn build_process_view(view: &mut SoulInspectorView, pid: i32, connections: &[Connection]) {
    view.has_selection = true;
    view.target_icon = "⚰️".to_string();
    view.pid = Some(pid);
    
    // Find connections for this process
    let process_conns: Vec<&Connection> = connections.iter()
        .filter(|c| c.pid == Some(pid))
        .collect();
    
    // Get process name from first connection
    let process_name = process_conns.iter()
        .find_map(|c| c.process_name.clone())
        .unwrap_or_else(|| format!("PID {}", pid));
    
    view.target_name = if process_name.len() > 15 {
        format!("{}...", &process_name[..12])
    } else {
        process_name.clone()
    };
    
    view.conn_count = process_conns.len();
    
    // Determine state based on connections
    let established = process_conns.iter().filter(|c| c.state == ConnectionState::Established).count();
    let listening = process_conns.iter().filter(|c| c.state == ConnectionState::Listen).count();
    let problematic = process_conns.iter()
        .filter(|c| matches!(c.state, ConnectionState::CloseWait | ConnectionState::TimeWait | ConnectionState::Close))
        .count();
    
    if process_conns.is_empty() {
        view.state_icon = "⚪".to_string();
        view.state_text = "No connections".to_string();
        view.state_color = BONE_WHITE;
    } else if problematic > 0 {
        view.state_icon = "🟠".to_string();
        view.state_text = format!("{} problematic", problematic);
        view.state_color = PUMPKIN_ORANGE;
    } else if established > 0 {
        view.state_icon = "🟢".to_string();
        view.state_text = format!("{} established", established);
        view.state_color = TOXIC_GREEN;
    } else if listening > 0 {
        view.state_icon = "🟡".to_string();
        view.state_text = format!("{} listening", listening);
        view.state_color = PUMPKIN_ORANGE;
    } else {
        view.state_icon = "⚪".to_string();
        view.state_text = "Idle".to_string();
        view.state_color = BONE_WHITE;
    }
    
    // Build socket list
    view.sockets = process_conns.iter()
        .take(5)
        .map(|c| connection_to_socket_info(c))
        .collect();
    
    // Tags
    view.tags.push(process_name);
    if listening > 0 {
        view.tags.push("server".to_string());
    }
    if established > 0 {
        view.tags.push("client".to_string());
    }
}

/// Convert Connection to SocketInfo for display
fn connection_to_socket_info(conn: &Connection) -> SocketInfo {
    let display = format!("tcp://{}:{}", conn.local_addr, conn.local_port);
    let remote = if conn.state == ConnectionState::Listen || conn.remote_addr == "0.0.0.0" {
        None
    } else {
        Some(format!("{}:{}", conn.remote_addr, conn.remote_port))
    };
    
    SocketInfo {
        display,
        remote,
        state: conn.state,
    }
}

/// Get display info for connection state
fn connection_state_display(state: ConnectionState) -> (String, String, Color) {
    match state {
        ConnectionState::Established => ("🟢".to_string(), "ESTABLISHED (Alive)".to_string(), TOXIC_GREEN),
        ConnectionState::Listen => ("🟡".to_string(), "LISTEN (Waiting)".to_string(), PUMPKIN_ORANGE),
        ConnectionState::TimeWait => ("🟠".to_string(), "TIME_WAIT (Closing)".to_string(), PUMPKIN_ORANGE),
        ConnectionState::CloseWait => ("🟠".to_string(), "CLOSE_WAIT (Stale)".to_string(), PUMPKIN_ORANGE),
        ConnectionState::Close => ("🔴".to_string(), "CLOSED (Dead)".to_string(), BLOOD_RED),
        ConnectionState::SynSent => ("🟡".to_string(), "SYN_SENT (Connecting)".to_string(), PUMPKIN_ORANGE),
        ConnectionState::SynRecv => ("🟡".to_string(), "SYN_RECV (Handshake)".to_string(), PUMPKIN_ORANGE),
        ConnectionState::FinWait1 | ConnectionState::FinWait2 => ("🟠".to_string(), "FIN_WAIT (Closing)".to_string(), PUMPKIN_ORANGE),
        ConnectionState::LastAck => ("🟠".to_string(), "LAST_ACK (Closing)".to_string(), PUMPKIN_ORANGE),
        ConnectionState::Closing => ("🟠".to_string(), "CLOSING".to_string(), PUMPKIN_ORANGE),
        ConnectionState::Unknown => ("⚪".to_string(), "UNKNOWN".to_string(), BONE_WHITE),
    }
}

/// Check for suspicious patterns in a connection
fn check_suspicious_patterns(view: &mut SoulInspectorView, conn: &Connection) {
    // High port to high port (potential C2)
    if conn.remote_port > 49152 && conn.local_port > 49152 {
        view.suspicious = true;
        view.tags.push("high-port".to_string());
    }
    
    // Connection to non-standard ports
    let standard_ports = [80, 443, 22, 21, 25, 53, 110, 143, 993, 995, 3306, 5432, 6379, 27017];
    if conn.state == ConnectionState::Established 
        && !standard_ports.contains(&conn.remote_port) 
        && conn.remote_port > 1024 
    {
        view.tags.push("non-standard".to_string());
    }
}

pub fn render_soul_inspector(f: &mut Frame, area: Rect, app: &AppState) {
    // Build view model from app state
    let view = build_soul_inspector_view(app);
    
    // Split area for content and sparkline
    let inspector_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11), // Top info with refresh rate
            Constraint::Length(5),  // Sparkline
            Constraint::Min(0),     // Socket list
        ])
        .split(area);

    // Check if refresh interval was recently changed
    let recently_changed = app.refresh_config.last_change
        .map(|last| last.elapsed() < crate::app::CHANGE_HIGHLIGHT_DURATION)
        .unwrap_or(false);

    // Get color for refresh interval based on its value
    let refresh_color = get_refresh_color(view.refresh_ms, 100, recently_changed);

    // Apply highlight style if recently changed
    let refresh_style = if recently_changed {
        Style::default().fg(refresh_color).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        Style::default().fg(refresh_color)
    };

    // Get status text based on overdrive mode
    let overdrive_enabled = app.graveyard_settings.overdrive_enabled;
    let overdrive_suffix = if overdrive_enabled { 
        get_status_text(ConnectionState::Established, true).to_string() 
    } else { 
        "Active".to_string() 
    };
    let status_display = format!("{} {} ({})", 
        view.state_icon, 
        view.state_text.split(" (").next().unwrap_or(&view.state_text),
        overdrive_suffix
    );

    // Build PID/PPID line
    let pid_line = if let Some(pid) = view.pid {
        Line::from(vec![
            Span::raw("  PID: "),
            Span::styled(pid.to_string(), Style::default().fg(Color::Cyan)),
            Span::raw("  |  Conns: "),
            Span::styled(view.conn_count.to_string(), Style::default().fg(Color::Gray)),
        ])
    } else {
        Line::from(vec![
            Span::raw("  Connections: "),
            Span::styled(view.conn_count.to_string(), Style::default().fg(Color::Cyan)),
        ])
    };

    // Build tags line if any
    let tags_line = if !view.tags.is_empty() {
        let tags_str = view.tags.iter()
            .take(3)
            .map(|t| format!("[{}]", t))
            .collect::<Vec<_>>()
            .join(" ");
        Line::from(vec![
            Span::raw("  "),
            Span::styled(tags_str, Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from("")
    };

    // Suspicious indicator
    let suspicious_indicator = if view.suspicious {
        Span::styled(" ⚠️", Style::default().fg(BLOOD_RED).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("")
    };

    // Top section with process info
    let top_content = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  TARGET: "),
            Span::styled(
                format!("{} {}", view.target_icon, view.target_name),
                Style::default()
                    .fg(PUMPKIN_ORANGE)
                    .add_modifier(Modifier::BOLD),
            ),
            suspicious_indicator,
        ]),
        pid_line,
        tags_line,
        Line::from(vec![
            Span::raw("  STATE: "),
            Span::styled(
                status_display,
                Style::default()
                    .fg(view.state_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  ⚡ Refresh: "),
            Span::styled(
                format!("{}ms", view.refresh_ms),
                refresh_style,
            ),
        ]),
    ];

    // Title with suspicious warning if applicable
    let title_spans = if view.suspicious {
        vec![
            Span::styled(
                "━ 🔮 Soul Inspector ",
                Style::default()
                    .fg(NEON_PURPLE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "⚠️ ",
                Style::default()
                    .fg(BLOOD_RED)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("━━━━", Style::default().fg(NEON_PURPLE)),
        ]
    } else {
        vec![
            Span::styled(
                "━ 🔮 Soul Inspector (Detail) ",
                Style::default()
                    .fg(NEON_PURPLE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("━━━━━━", Style::default().fg(NEON_PURPLE)),
        ]
    };

    let top_paragraph = Paragraph::new(top_content).block(
        Block::default()
            .title(title_spans)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(if view.suspicious { BLOOD_RED } else { NEON_PURPLE })),
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

    // Bottom section with socket list - now using real data
    let mut socket_lines = vec![Line::from("")];

    if view.sockets.is_empty() {
        socket_lines.push(Line::from(vec![
            Span::styled(
                "  (no sockets)",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]));
    } else {
        for socket in &view.sockets {
            let state_color = match socket.state {
                ConnectionState::Established => TOXIC_GREEN,
                ConnectionState::Listen => PUMPKIN_ORANGE,
                ConnectionState::TimeWait | ConnectionState::CloseWait => PUMPKIN_ORANGE,
                ConnectionState::Close => BLOOD_RED,
                _ => BONE_WHITE,
            };

            let state_str = match socket.state {
                ConnectionState::Established => "ESTABLISHED",
                ConnectionState::Listen => "LISTEN",
                ConnectionState::TimeWait => "TIME_WAIT",
                ConnectionState::CloseWait => "CLOSE_WAIT",
                ConnectionState::Close => "CLOSED",
                ConnectionState::SynSent => "SYN_SENT",
                _ => "OTHER",
            };

            if let Some(ref remote) = socket.remote {
                socket_lines.push(Line::from(vec![
                    Span::raw("  > "),
                    Span::styled(&socket.display, Style::default().fg(Color::Cyan)),
                    Span::raw(" → "),
                    Span::styled(remote, Style::default().fg(Color::Blue)),
                ]));
            } else {
                socket_lines.push(Line::from(vec![
                    Span::raw("  > "),
                    Span::styled(&socket.display, Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!(" ({})", state_str),
                        Style::default().fg(state_color),
                    ),
                ]));
            }
        }

        // Show "and N more" if there are more sockets
        if view.conn_count > view.sockets.len() {
            socket_lines.push(Line::from(vec![Span::styled(
                format!("  ... and {} more", view.conn_count - view.sockets.len()),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )]));
        }
    }

    let socket_paragraph = Paragraph::new(socket_lines).block(
        Block::default()
            .title(vec![Span::styled(
                format!(" 📜 Open Sockets ({}) ", view.sockets.len()),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )])
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(NEON_PURPLE)),
    );

    f.render_widget(socket_paragraph, inspector_chunks[2]);
}
