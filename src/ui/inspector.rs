// Soul Inspector rendering module
//
// Renders the detail panel showing selected process/connection information,
// traffic sparkline, and socket list.

use crate::app::AppState;
use crate::net::ConnectionState;
use crate::theme::{get_refresh_color, get_status_text, NEON_PURPLE, PUMPKIN_ORANGE, TOXIC_GREEN};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Sparkline},
    Frame,
};

pub fn render_soul_inspector(f: &mut Frame, area: Rect, app: &AppState) {
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
    let refresh_color = get_refresh_color(app.refresh_config.refresh_ms, 100, recently_changed);

    // Apply highlight style if recently changed
    let refresh_style = if recently_changed {
        Style::default().fg(refresh_color).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        Style::default().fg(refresh_color)
    };

    // Get status text based on overdrive mode
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
