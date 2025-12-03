// UI rendering module

use crate::app::AppState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, List, ListItem, Paragraph},
    Frame,
};

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

fn render_network_map(f: &mut Frame, area: Rect, _app: &AppState) {
    let map_content = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("          "),
            Span::styled("(☁️ AWS-Cloud-LB)", Style::default().fg(Color::Cyan)),
        ]),
        Line::from("                │"),
        Line::from(vec![
            Span::raw("                │ "),
            Span::styled("⡠⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⢄", Style::default().fg(Color::Rgb(138, 43, 226))),
            Span::styled(" <SSL/443>", Style::default().fg(Color::Rgb(138, 43, 226))),
            Span::styled(" (🟣 Solid Neon)", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::raw("                ▼   "),
            Span::styled("⠱⡀", Style::default().fg(Color::Rgb(138, 43, 226))),
            Span::styled("   12ms      ⠑⢄", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::raw("                     "),
            Span::styled("⠱⡀", Style::default().fg(Color::Rgb(138, 43, 226))),
            Span::raw("              ⠑⢄       "),
            Span::styled("[🧟 zombie-proc]", Style::default().fg(Color::Red)),
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

fn render_soul_inspector(f: &mut Frame, area: Rect, _app: &AppState) {
    let inspection_content = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  TARGET: "),
            Span::styled("⚰️ kafka-broker-1", Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)),
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
            Span::styled("🟢 ESTABLISHED (Alive)", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [📊 Traffic History (Last 1m)]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  In :  "),
            Span::styled("█▇▆▅▄▃▂", Style::default().fg(Color::Green)),
            Span::raw("     "),
            Span::styled("(2.5 MB/s)", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::raw("  Out:  "),
            Span::styled("████▇▆▅", Style::default().fg(Color::Blue)),
            Span::raw("     "),
            Span::styled("(4.1 MB/s)", Style::default().fg(Color::Blue)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [📜 Open Sockets List]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("  > "),
            Span::styled("tcp://0.0.0.0:9092", Style::default().fg(Color::Cyan)),
            Span::styled(" (LISTEN)", Style::default().fg(Color::Green)),
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
            Span::styled(" (ESTABLISHED)", Style::default().fg(Color::Green)),
        ]),
    ];

    let inspection = Paragraph::new(inspection_content)
        .block(
            Block::default()
                .title(vec![
                    Span::styled("━ 🔮 Soul Inspector (Detail) ", Style::default().fg(Color::Rgb(138, 43, 226)).add_modifier(Modifier::BOLD)),
                    Span::styled("━━━━━━", Style::default().fg(Color::Rgb(138, 43, 226))),
                ])
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(138, 43, 226)))
        );

    f.render_widget(inspection, area);
}

fn render_grimoire(f: &mut Frame, area: Rect, _app: &AppState) {
    let log_items = vec![
        ListItem::new(Line::from(vec![
            Span::styled(" [14:20:55] ", Style::default().fg(Color::DarkGray)),
            Span::styled("ℹ️", Style::default().fg(Color::Cyan)),
            Span::raw(" New spirit summoned:"),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("            "),
            Span::styled("[nginx-gw]", Style::default().fg(Color::Green)),
            Span::styled(" (PID 8821)", Style::default().fg(Color::Gray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(" [14:21:10] ", Style::default().fg(Color::DarkGray)),
            Span::styled("⚠️", Style::default().fg(Color::Yellow)),
            Span::raw(" High latency ritual:"),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("            "),
            Span::styled("MAIN_APP", Style::default().fg(Color::Rgb(255, 140, 0))),
            Span::raw(" -> "),
            Span::styled("auth-svc", Style::default().fg(Color::Cyan)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(" [14:21:45] ", Style::default().fg(Color::DarkGray)),
            Span::styled("🔴", Style::default().fg(Color::Red)),
            Span::styled(" ZOMBIE PROCESS DETECTED:", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("            "),
            Span::styled("[zombie-proc]", Style::default().fg(Color::Red)),
            Span::styled(" (PID 999)", Style::default().fg(Color::Gray)),
        ])),
    ];

    let logs = List::new(log_items)
        .block(
            Block::default()
                .title(vec![
                    Span::styled("━ 📜 Grimoire (Logs & Alerts) ", Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)),
                    Span::styled("━━━━━━━", Style::default().fg(Color::Rgb(255, 140, 0))),
                ])
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(255, 140, 0)))
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
