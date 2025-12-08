// Default theme functions
//
// This module provides the standard (non-overdrive) theme functions for ntomb.
// These functions return professional, clear status text and colors.
//
// Requirements: 2.2

use ratatui::style::Color;

use super::{BLOOD_RED, PUMPKIN_ORANGE, TOXIC_GREEN};
use crate::net::ConnectionState;

/// Get the normal (non-overdrive) status text for a connection state
///
/// Returns standard, professional status descriptions for connection states.
/// Used when Kiroween Overdrive mode is disabled.
///
/// # Arguments
/// * `state` - The connection state
///
/// # Returns
/// A static string containing the standard status text
pub fn get_normal_status_text(state: ConnectionState) -> &'static str {
    match state {
        ConnectionState::Established => "Alive",
        ConnectionState::Listen => "Listening",
        ConnectionState::TimeWait
        | ConnectionState::CloseWait
        | ConnectionState::FinWait1
        | ConnectionState::FinWait2
        | ConnectionState::LastAck
        | ConnectionState::Closing
        | ConnectionState::Close => "Closing",
        ConnectionState::SynSent | ConnectionState::SynRecv => "Connecting",
        ConnectionState::Unknown => "Unknown",
    }
}

/// Interpolate between two RGB colors based on a ratio (0.0 ~ 1.0)
///
/// # Arguments
/// * `color1` - Starting color as (r, g, b) tuple
/// * `color2` - Ending color as (r, g, b) tuple
/// * `ratio` - Interpolation ratio (0.0 = color1, 1.0 = color2)
///
/// # Returns
/// Interpolated Color::Rgb value
pub fn interpolate_color(color1: (u8, u8, u8), color2: (u8, u8, u8), ratio: f32) -> Color {
    let ratio = ratio.clamp(0.0, 1.0);
    let r = (color1.0 as f32 + (color2.0 as f32 - color1.0 as f32) * ratio) as u8;
    let g = (color1.1 as f32 + (color2.1 as f32 - color1.1 as f32) * ratio) as u8;
    let b = (color1.2 as f32 + (color2.2 as f32 - color1.2 as f32) * ratio) as u8;
    Color::Rgb(r, g, b)
}

/// Get color for refresh interval based on its value relative to default
///
/// Color coding:
/// - Green (TOXIC_GREEN): Default value (normal performance impact)
/// - Yellow (PUMPKIN_ORANGE): High frequency (increased performance impact)
/// - Red (BLOOD_RED): Very high frequency (significant performance impact)
///
/// If recently_changed is true, returns a brighter version of the color
///
/// # Arguments
/// * `interval_ms` - Current refresh interval in milliseconds
/// * `default_ms` - Default refresh interval in milliseconds
/// * `recently_changed` - Whether the value was recently changed (triggers highlight)
///
/// # Returns
/// Color appropriate for the refresh interval state
pub fn get_refresh_color(interval_ms: u64, default_ms: u64, recently_changed: bool) -> Color {
    let base_color = if interval_ms == default_ms {
        // Default value - green
        TOXIC_GREEN
    } else if interval_ms < default_ms {
        // Faster than default (higher frequency)
        let ratio = (default_ms - interval_ms) as f32 / default_ms as f32;

        if ratio > 0.5 {
            // Very high frequency - red
            BLOOD_RED
        } else {
            // High frequency - yellow/orange
            PUMPKIN_ORANGE
        }
    } else {
        // Slower than default - also use green (lower resource usage)
        TOXIC_GREEN
    };

    // If recently changed, make the color brighter
    if recently_changed {
        match base_color {
            Color::Rgb(r, g, b) => {
                // Increase brightness by 20%
                let r = ((r as f32 * 1.2).min(255.0)) as u8;
                let g = ((g as f32 * 1.2).min(255.0)) as u8;
                let b = ((b as f32 * 1.2).min(255.0)) as u8;
                Color::Rgb(r, g, b)
            }
            _ => base_color,
        }
    } else {
        base_color
    }
}
