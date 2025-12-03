// UI rendering module

use crate::app::AppState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, List, ListItem, Paragraph, Sparkline},
    Frame,
};

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
    render_status_bar(f, chunks[2]);
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

fn render_network_map(f: &mut Frame, area: Rect, app: &AppState) {
    // Calculate pulsing neon color based on pulse_phase
    let pulse_color = interpolate_color((138, 43, 226), (187, 154, 247), app.pulse_phase);

    // Zombie color based on blink state
    let zombie_color = if app.zombie_blink {
        BLOOD_RED
    } else {
        Color::Rgb(100, 60, 70) // Faded red
    };
    let map_content = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("          "),
            Span::styled("(☁️ AWS-Cloud-LB)", Style::default().fg(Color::Cyan)),
        ]),
        Line::from("                │"),
        Line::from(vec![
            Span::raw("                │ "),
            Span::styled("⡠⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⢄", Style::default().fg(pulse_color)),
            Span::styled(" <SSL/443>", Style::default().fg(pulse_color)),
            Span::styled(" (🟣 Pulsing)", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::raw("                ▼   "),
            Span::styled("⠱⡀", Style::default().fg(pulse_color)),
            Span::styled("   12ms      ⠑⢄", Style::default().fg(TOXIC_GREEN)),
        ]),
        Line::from(vec![
            Span::raw("                     "),
            Span::styled("⠱⡀", Style::default().fg(pulse_color)),
            Span::raw("              ⠑⢄       "),
            Span::styled("[🧟 zombie-proc]", Style::default().fg(zombie_color)),
        ]),
        Line::from(vec![
            Span::raw("                      "),
            Span::styled("⢣", Style::default().fg(Color::Rgb(138, 43, 226))),
            Span::raw("               ▼          "),
            Span::styled("<TCP/???>", Style::default().fg(Color::Red)),
        ]),
        Line::from(vec![
            Span::raw("                      "),
            Span::styled("⢣", Style::default().fg(Color::Rgb(138, 43, 226))),
            Span::raw("   "),
            Span::styled("╭─────────────╮", Style::default().fg(Color::Rgb(255, 140, 0))),
            Span::raw("        "),
            Span::styled("⡠⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤", Style::default().fg(Color::Red)),
        ]),
        Line::from(vec![
            Span::styled("      <TCP/80>", Style::default().fg(Color::Green)),
            Span::raw("        "),
            Span::styled("⢣", Style::default().fg(Color::Rgb(138, 43, 226))),
            Span::raw("  "),
            Span::styled("│ ⚰️ MAIN_APP │", Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)),
            Span::raw("        :  "),
            Span::styled("⚠️", Style::default().fg(Color::Red)),
            Span::styled(" (🔴 Dotted/Flash)", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("      [nginx-gw]", Style::default().fg(Color::Green)),
            Span::raw(" ⠀⠀⠀ ⠈⠒⠚⠁"),
            Span::styled("│  (PID 1337) │", Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)),
            Span::styled("◀⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒", Style::default().fg(Color::Red)),
        ]),
        Line::from(vec![
            Span::styled("         🎃", Style::default().fg(Color::Rgb(255, 165, 0))),
            Span::raw("           "),
            Span::styled("⢣", Style::default().fg(Color::Rgb(138, 43, 226))),
            Span::raw("  "),
            Span::styled("╰─────────────╯", Style::default().fg(Color::Rgb(255, 140, 0))),
            Span::raw("        :   WAIT_CLOSE"),
        ]),
        Line::from(vec![
            Span::raw("        ╱ "),
            Span::styled("(🟢)", Style::default().fg(Color::Green)),
            Span::raw("          "),
            Span::styled("⢣", Style::default().fg(Color::Rgb(138, 43, 226))),
            Span::raw("       │                :"),
        ]),
        Line::from(vec![
            Span::raw("       ╱                "),
            Span::styled("⢣", Style::default().fg(Color::Rgb(138, 43, 226))),
            Span::raw("      │ "),
            Span::styled("<TCP/5432>", Style::default().fg(Color::Rgb(138, 43, 226))),
            Span::raw("     "),
            Span::styled("⠑⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤", Style::default().fg(Color::Rgb(138, 43, 226))),
        ]),
        Line::from(vec![
            Span::raw("      ╱ "),
            Span::styled("(🟠 High Lat)", Style::default().fg(Color::Rgb(255, 140, 0))),
            Span::raw("    "),
            Span::styled("⠱⡀", Style::default().fg(Color::Rgb(138, 43, 226))),
            Span::raw("   │ "),
            Span::styled("(🟣 Solid Neon)", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::raw("     ▼ "),
            Span::styled("450ms", Style::default().fg(Color::Yellow)),
            Span::raw("             "),
            Span::styled("⠱⡀", Style::default().fg(Color::Rgb(138, 43, 226))),
            Span::raw("  ▼"),
        ]),
        Line::from(vec![
            Span::styled("  [💳 auth-svc]", Style::default().fg(Color::Cyan)),
            Span::raw("             "),
            Span::styled("⠱", Style::default().fg(Color::Rgb(138, 43, 226))),
            Span::raw(" "),
            Span::styled("[🪦 postgres-db]", Style::default().fg(Color::Blue)),
        ]),
        Line::from(vec![
            Span::raw("                              "),
            Span::styled("(Tombstone)", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("   * RENDER NOTE:", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
            Span::styled(" Curves (`⡠⠤⢄`) are drawn using Ratatui Canvas Braille", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("     resolution (2x4 pixels per cell) for smooth, organic visuals.", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let network_map = Paragraph::new(map_content)
        .block(
            Block::default()
                .title(vec![
                    Span::styled("━ 🕸️ The Graveyard (Network Topology) ━", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
                    Span::styled("[🔍 Zoom: 120% | 📍 Offset: (45, 22)]", Style::default().fg(Color::Gray)),
                    Span::styled("━━━━", Style::default().fg(Color::Rgb(138, 43, 226))),
                ])
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(138, 43, 226)))
        )
        .alignment(Alignment::Left);

    f.render_widget(network_map, area);
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

        log_items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{:2}.", idx + 1), Style::default().fg(Color::DarkGray)),
            Span::styled(conn_line, Style::default().fg(state_color)),
        ])));
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

fn render_status_bar(f: &mut Frame, area: Rect) {
    let status_text = Line::from(vec![
        Span::styled(" 💀 ", Style::default().fg(Color::Rgb(138, 43, 226))),
        Span::styled("F1:", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
        Span::raw("Help | "),
        Span::styled("⇆ TAB:", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
        Span::raw("Switch Pane | "),
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
