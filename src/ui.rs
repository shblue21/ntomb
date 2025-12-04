// UI rendering module

use crate::app::AppState;
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

/// Interpolate between two RGB colors based on a ratio (0.0 ~ 1.0)
fn interpolate_color(color1: (u8, u8, u8), color2: (u8, u8, u8), ratio: f32) -> Color {
    let ratio = ratio.clamp(0.0, 1.0);
    let r = (color1.0 as f32 + (color2.0 as f32 - color1.0 as f32) * ratio) as u8;
    let g = (color1.1 as f32 + (color2.1 as f32 - color1.1 as f32) * ratio) as u8;
    let b = (color1.2 as f32 + (color2.2 as f32 - color1.2 as f32) * ratio) as u8;
    Color::Rgb(r, g, b)
}

/// Main UI drawing function
pub fn draw(f: &mut Frame, app: &AppState) {
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
    render_banner(f, chunks[0]);

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

fn render_banner(f: &mut Frame, area: Rect) {
    let banner_text = vec![
        Line::from(""),
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
            Span::styled("   [💀 Total Souls: 128] [🩸 BPF Radar: ACTIVE]", Style::default().fg(Color::Red)),
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
struct EndpointNode {
    label: String,
    x: f64,
    y: f64,
    state: ConnectionState,
    conn_count: usize,
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
    let center_label = match app.graveyard_mode {
        GraveyardMode::Host => "⚰️ HOST".to_string(),
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
                
                format!("⚰️ PROC: {} ({})", process_name, pid)
            } else {
                "⚰️ HOST".to_string()
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

    // Prepare endpoint nodes with radial layout
    let mut sorted_endpoints: Vec<_> = endpoints_map.iter().collect();
    sorted_endpoints.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    let max_nodes = 12;
    let nodes: Vec<EndpointNode> = sorted_endpoints
        .iter()
        .take(max_nodes)
        .enumerate()
        .map(|(i, (addr, conns))| {
            // Calculate radial position
            let angle = (i as f64 / max_nodes as f64) * 2.0 * std::f64::consts::PI - std::f64::consts::PI / 2.0;
            let radius = 35.0;
            let x = 50.0 + radius * angle.cos();
            let y = 50.0 + radius * angle.sin();

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
                addr.to_string()
            };

            EndpointNode {
                label,
                x,
                y,
                state,
                conn_count: conns.len(),
            }
        })
        .collect();

    // Pulsing color for animation
    let pulse_color = interpolate_color((138, 43, 226), (187, 154, 247), app.pulse_phase);

    // Capture values for closure
    let is_empty = nodes.is_empty() && filtered_connections.is_empty();
    let graveyard_mode = app.graveyard_mode;

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

            // Draw connection lines first (behind nodes)
            for node in &nodes {
                let line_color = match node.state {
                    ConnectionState::Established => TOXIC_GREEN,
                    ConnectionState::TimeWait | ConnectionState::CloseWait => PUMPKIN_ORANGE,
                    ConnectionState::SynSent | ConnectionState::SynRecv => Color::Yellow,
                    ConnectionState::Close => BLOOD_RED,
                    _ => pulse_color,
                };

                ctx.draw(&CanvasLine {
                    x1: cx,
                    y1: cy,
                    x2: node.x,
                    y2: node.y,
                    color: line_color,
                });
            }

            // Draw central node with mode-specific label (Requirement 5.1)
            let label_offset = (center_label.len() as f64 / 2.0) * 1.2;
            ctx.print(cx - label_offset, cy + 2.0, Span::styled(center_label.clone(), Style::default().fg(PUMPKIN_ORANGE).add_modifier(Modifier::BOLD)));

            // Draw endpoint nodes
            for node in &nodes {
                let icon = match node.state {
                    ConnectionState::Established => "🎃",
                    ConnectionState::TimeWait => "👻",
                    ConnectionState::CloseWait => "💀",
                    ConnectionState::SynSent => "⏳",
                    ConnectionState::Listen => "👂",
                    _ => "🌐",
                };

                let color = match node.state {
                    ConnectionState::Established => TOXIC_GREEN,
                    ConnectionState::TimeWait | ConnectionState::CloseWait => PUMPKIN_ORANGE,
                    ConnectionState::Close => BLOOD_RED,
                    _ => BONE_WHITE,
                };

                // Node icon
                ctx.print(node.x, node.y, Span::styled(icon, Style::default().fg(color)));

                // Node label (shortened)
                let label = format!("{} ({})", node.label, node.conn_count);
                ctx.print(
                    node.x - 6.0,
                    node.y - 4.0,
                    Span::styled(label, Style::default().fg(color)),
                );
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

            // Show "more" indicator
            if sorted_endpoints.len() > max_nodes {
                ctx.print(
                    cx - 8.0,
                    10.0,
                    Span::styled(
                        format!("+{} more", sorted_endpoints.len() - max_nodes),
                        Style::default().fg(Color::DarkGray),
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
            Constraint::Length(10), // Top info
            Constraint::Length(5),  // Sparkline
            Constraint::Min(0),     // Socket list
        ])
        .split(area);

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
                "🟢 ESTABLISHED (Alive)",
                Style::default()
                    .fg(TOXIC_GREEN)
                    .add_modifier(Modifier::BOLD),
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

fn render_grimoire(f: &mut Frame, area: Rect, app: &AppState) {
    use crate::net::ConnectionState;

    let mut log_items = Vec::new();

    // Show connection error if any
    if let Some(ref error) = app.conn_error {
        log_items.push(ListItem::new(Line::from(vec![
            Span::styled(" ℹ️ ", Style::default().fg(Color::Cyan)),
            Span::styled(error, Style::default().fg(BONE_WHITE)),
        ])));
    } else {
        // Show connection count
        log_items.push(ListItem::new(Line::from(vec![
            Span::styled(" 🔗 ", Style::default().fg(NEON_PURPLE)),
            Span::styled(
                format!("Active Connections: {}", app.connections.len()),
                Style::default().fg(TOXIC_GREEN).add_modifier(Modifier::BOLD),
            ),
        ])));
    }

    // Show top 15 connections
    for (idx, conn) in app.connections.iter().take(15).enumerate() {
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

    // Show "..." if there are more connections
    if app.connections.len() > 15 {
        log_items.push(ListItem::new(Line::from(vec![Span::styled(
            format!(" ... and {} more", app.connections.len() - 15),
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )])));
    }

    let logs = List::new(log_items).block(
        Block::default()
            .title(vec![
                Span::styled(
                    "━ 🌐 Active Connections ",
                    Style::default()
                        .fg(PUMPKIN_ORANGE)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("━━━━━━━", Style::default().fg(PUMPKIN_ORANGE)),
            ])
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(PUMPKIN_ORANGE)),
    );

    f.render_widget(logs, area);
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &AppState) {
    use crate::app::GraveyardMode;
    
    // Determine mode-specific hint text (Requirements 7.1, 7.2)
    let mode_hint = match app.graveyard_mode {
        GraveyardMode::Host => "P:Focus Process",
        GraveyardMode::Process => "P:Back to Host",
    };
    
    let status_text = Line::from(vec![
        Span::styled(" 💀 ", Style::default().fg(Color::Rgb(138, 43, 226))),
        Span::styled("F1:", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
        Span::raw("Help | "),
        Span::styled("⇆ TAB:", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
        Span::raw("Switch Pane | "),
        Span::styled("P:", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
        Span::raw(mode_hint),
        Span::raw(" | "),
        Span::styled("🖱️ Drag:", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
        Span::raw("Pan Map | "),
        Span::styled("➕/➖", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
        Span::raw(" Zoom | "),
        Span::styled("❌ X:", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw("Exorcise(Kill) | "),
        Span::styled("Q:", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw("R.I.P "),
    ]);

    let status_bar = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Rgb(138, 43, 226)))
        )
        .alignment(Alignment::Left);

    f.render_widget(status_bar, area);
}
