// Banner rendering module
//
// Renders the top banner with ASCII art logo and global stats.

use crate::app::AppState;
use crate::theme::get_stats_label;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

pub fn render_banner(f: &mut Frame, area: Rect, app: &AppState) {
    // Get the appropriate stats label based on overdrive mode (Requirement 4.5)
    // When overdrive is enabled, use "Spirits" instead of "Total Souls"
    let stats_label = get_stats_label(app.graveyard_settings.overdrive_enabled);
    let conn_count = app.connections.len();
    let stats_text = format!(
        "   [💀 {}: {}] [🩸 BPF Radar: TBD]",
        stats_label, conn_count
    );

    let banner_text = vec![
        Line::from(vec![Span::styled(
            "   _   _  _____  ____   __  __  ____  ",
            Style::default()
                .fg(Color::Rgb(138, 43, 226))
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled(
                "  | \\ | ||_   _|/ __ \\ |  \\/  ||  _ \\ ",
                Style::default().fg(Color::Rgb(148, 53, 236)),
            ),
            Span::styled(
                "   >>> The Necromancer's Terminal v0.0.1 <<<",
                Style::default()
                    .fg(Color::Rgb(255, 140, 0))
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  |  \\| |  | | | |  | || |\\/| || |_) |",
                Style::default().fg(Color::Rgb(158, 63, 246)),
            ),
            Span::styled(
                "   \"Revealing the unseen connections of the undead.\"",
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(vec![Span::styled(
            "  | |\\  |  | | | |__| || |  | || |_) |",
            Style::default().fg(Color::Rgb(168, 73, 255)),
        )]),
        Line::from(vec![
            Span::styled(
                "  |_| \\_|  |_|  \\____/ |_|  |_||____/ ",
                Style::default().fg(Color::Rgb(178, 83, 255)),
            ),
            Span::styled(stats_text, Style::default().fg(Color::Red)),
        ]),
    ];

    let banner = Paragraph::new(banner_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Rgb(138, 43, 226))),
        )
        .alignment(Alignment::Left);

    f.render_widget(banner, area);
}
