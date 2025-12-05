// Grimoire (Connection List) rendering module
//
// Renders the scrollable list of active network connections with
// state-based coloring and process information.

use crate::app::AppState;
use crate::net::ConnectionState;
use crate::theme::{BLOOD_RED, BONE_WHITE, PUMPKIN_ORANGE, TOXIC_GREEN};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};

pub fn render_grimoire(f: &mut Frame, area: Rect, app: &mut AppState) {
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

        // Add process info tag if available
        let process_tag = if let (Some(pid), Some(ref name)) = (conn.pid, &conn.process_name) {
            format!(" [{}({})]", name, pid)
        } else {
            String::new()
        };

        // Check if this connection is selected
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
