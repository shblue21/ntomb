// Theme module - Color constants and theme re-exports
//
// This module provides the color palette and theme functions for ntomb's
// Halloween-themed UI. Colors follow the "Witching Hour" theme from
// ntomb-visual-design.md.
//
// Requirements: 2.1

pub mod default;
pub mod overdrive;

use ratatui::style::Color;

// Color constants from ntomb-visual-design.md
// These define the core palette used throughout the UI

/// Primary accent color - used for borders, titles, normal connections
/// RGB: (187, 154, 247)
pub const NEON_PURPLE: Color = Color::Rgb(187, 154, 247);

/// Warning/latency indicator - used for high latency, degraded states
/// RGB: (255, 158, 100)
pub const PUMPKIN_ORANGE: Color = Color::Rgb(255, 158, 100);

/// Danger/zombie indicator - used for errors, broken connections
/// RGB: (247, 118, 142)
pub const BLOOD_RED: Color = Color::Rgb(247, 118, 142);

/// Active/healthy indicator - used for alive states, new connections
/// RGB: (158, 206, 106)
pub const TOXIC_GREEN: Color = Color::Rgb(158, 206, 106);

/// Inactive/neutral text - used for general text, inactive nodes
/// RGB: (169, 177, 214)
pub const BONE_WHITE: Color = Color::Rgb(169, 177, 214);

// Re-export theme functions for convenient access
pub use default::*;
pub use overdrive::*;
