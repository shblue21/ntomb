// Status Bar rendering module
//
// Renders the bottom status bar with keyboard shortcuts and toggle indicators.

use crate::app::{AppState, GraveyardMode};
use crate::theme::{BONE_WHITE, NEON_PURPLE, TOXIC_GREEN};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

pub fn render_status_bar(f: &mut Frame, area: Rect, app: &AppState) {
    // Determine mode-specific hint text
    let mode_hint = match app.graveyard_mode {
        GraveyardMode::Host => "Focus Process | ",
        GraveyardMode::Process => "Back to Host | ",
    };

    // Calculate available width for hints (subtract borders and icon)
    let available_width = area.width.saturating_sub(4);

    // Define all hints with priority levels
    struct Hint {
        priority: u8,
        key: &'static str,
        desc: String,
        color: Color,
    }

    let hints = vec![
        Hint {
            priority: 1,
            key: "Q:",
            desc: "R.I.P ".to_string(),
            color: Color::Red,
        },
        Hint {
            priority: 1,
            key: "↑↓:",
            desc: "Navigate | ".to_string(),
            color: NEON_PURPLE,
        },
        Hint {
            priority: 1,
            key: "P:",
            desc: mode_hint.to_string(),
            color: NEON_PURPLE,
        },
        Hint {
            priority: 2,
            key: "+/-:",
            desc: "Speed | ".to_string(),
            color: NEON_PURPLE,
        },
        Hint {
            priority: 2,
            key: "A:",
            desc: "Anim | ".to_string(),
            color: NEON_PURPLE,
        },
        Hint {
            priority: 2,
            key: "H:",
            desc: "Theme | ".to_string(),
            color: NEON_PURPLE,
        },
        Hint {
            priority: 2,
            key: "t:",
            desc: "Labels | ".to_string(),
            color: NEON_PURPLE,
        },
        Hint {
            priority: 3,
            key: "F1:",
            desc: "Help | ".to_string(),
            color: NEON_PURPLE,
        },
    ];

    // Build status text, adding hints until we run out of space
    let mut spans = vec![Span::styled(" 💀 ", Style::default().fg(NEON_PURPLE))];

    let mut current_length = 4;

    // Process hints by priority
    for priority in 1..=3 {
        for hint in &hints {
            if hint.priority == priority {
                let hint_length = hint.key.len() + hint.desc.len();
                if current_length + hint_length <= available_width as usize {
                    spans.push(Span::styled(
                        hint.key,
                        Style::default().fg(hint.color).add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::raw(hint.desc.clone()));
                    current_length += hint_length;
                }
            }
        }
    }

    // Add toggle status indicators (always show, they're important for debugging)
    let toggle_indicators = build_toggle_indicators(app);
    spans.push(Span::raw(" "));
    spans.extend(toggle_indicators);

    let status_text = Line::from(spans);

    let status_bar = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(NEON_PURPLE)),
        )
        .alignment(Alignment::Left);

    f.render_widget(status_bar, area);
}

/// Build toggle status indicator spans for the status bar
/// Shows [A:ON/OFF] [H:ON/OFF] [t:ON/OFF] with appropriate colors
/// Toxic Green for ON, Bone White for OFF
pub fn build_toggle_indicators(app: &AppState) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    // Animation toggle [A:ON/OFF]
    let anim_state = if app.graveyard_settings.animations_enabled {
        "ON"
    } else {
        "OFF"
    };
    let anim_color = if app.graveyard_settings.animations_enabled {
        TOXIC_GREEN
    } else {
        BONE_WHITE
    };
    spans.push(Span::styled("[A:", Style::default().fg(BONE_WHITE)));
    spans.push(Span::styled(
        anim_state,
        Style::default().fg(anim_color).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled("] ", Style::default().fg(BONE_WHITE)));

    // Overdrive/Theme toggle [H:ON/OFF]
    let overdrive_state = if app.graveyard_settings.overdrive_enabled {
        "ON"
    } else {
        "OFF"
    };
    let overdrive_color = if app.graveyard_settings.overdrive_enabled {
        TOXIC_GREEN
    } else {
        BONE_WHITE
    };
    spans.push(Span::styled("[H:", Style::default().fg(BONE_WHITE)));
    spans.push(Span::styled(
        overdrive_state,
        Style::default()
            .fg(overdrive_color)
            .add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled("] ", Style::default().fg(BONE_WHITE)));

    // Labels toggle [t:ON/OFF]
    let labels_state = if app.graveyard_settings.labels_enabled {
        "ON"
    } else {
        "OFF"
    };
    let labels_color = if app.graveyard_settings.labels_enabled {
        TOXIC_GREEN
    } else {
        BONE_WHITE
    };
    spans.push(Span::styled("[t:", Style::default().fg(BONE_WHITE)));
    spans.push(Span::styled(
        labels_state,
        Style::default()
            .fg(labels_color)
            .add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled("] ", Style::default().fg(BONE_WHITE)));

    // Emoji width offset indicator [E:±N]
    // Shows current emoji width offset for cross-platform debugging
    let offset = app.graveyard_settings.emoji_width_offset;
    let offset_str = if offset >= 0 {
        format!("+{}", offset)
    } else {
        format!("{}", offset)
    };
    spans.push(Span::styled("[E:", Style::default().fg(BONE_WHITE)));
    spans.push(Span::styled(
        offset_str,
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled("]", Style::default().fg(BONE_WHITE)));

    spans
}
