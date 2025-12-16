// Application configuration types
//
// This module contains configuration structs and enums for:
// - Graveyard visual settings
// - Latency thresholds
// - Refresh intervals
// - View modes

use std::time::{Duration, Instant};

// ============================================================================
// Constants
// ============================================================================

/// Minimum refresh interval in milliseconds
pub const MIN_REFRESH_MS: u64 = 50;

/// Maximum refresh interval in milliseconds
pub const MAX_REFRESH_MS: u64 = 10000;

/// Refresh interval adjustment step in milliseconds
pub const REFRESH_STEP: u64 = 50;

/// Data refresh multiplier (data refreshes at N times the UI interval)
pub const DATA_REFRESH_MULTIPLIER: u64 = 10;

/// Duration to highlight recently changed refresh intervals
pub const CHANGE_HIGHLIGHT_DURATION: Duration = Duration::from_millis(500);

/// Tick interval for pulse animation (100ms)
pub const TICK_INTERVAL_MS: u128 = 100;

/// Blink interval for zombie animation (500ms)
pub const BLINK_INTERVAL_MS: u128 = 500;

/// Frame time threshold for auto-reducing animation complexity (100ms)
/// If frame time consistently exceeds this, particle count is reduced
pub const FRAME_TIME_THRESHOLD_MS: u128 = 100;

/// Number of consecutive slow frames before triggering complexity reduction
pub const SLOW_FRAME_COUNT_THRESHOLD: u32 = 5;

/// Number of log entries in the grimoire (for bounds checking)
#[allow(dead_code)]
pub const LOG_ENTRY_COUNT: usize = 6;

// ============================================================================
// Enums
// ============================================================================

/// Graveyard view mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GraveyardMode {
    /// Host-wide view (default)
    #[default]
    Host,
    /// Selected process view
    Process,
}

/// Latency bucket classification for ring positioning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LatencyBucket {
    /// Low latency (< 50ms) - innermost ring
    Low,
    /// Medium latency (50-200ms) - middle ring
    Medium,
    /// High latency (> 200ms) - outermost ring
    High,
    /// No latency data available - use default position
    Unknown,
}

// ============================================================================
// Configuration Structs
// ============================================================================

/// Visual settings for the Graveyard panel
/// Controls animations, labels, and theme enhancements
#[derive(Debug, Clone)]
pub struct GraveyardSettings {
    /// Enable particle animations on edges (toggle with 'A' key)
    pub animations_enabled: bool,

    /// Show text labels on endpoints (toggle with 't' key)
    pub labels_enabled: bool,

    /// Enable Kiroween Overdrive theme (toggle with 'H' key)
    pub overdrive_enabled: bool,

    /// Emoji width offset for cross-platform rendering correction
    /// Positive: emoji renders wider than expected
    /// Negative: emoji renders narrower than expected
    /// Adjust with '[' and ']' keys
    pub emoji_width_offset: i32,
}

impl Default for GraveyardSettings {
    fn default() -> Self {
        Self {
            animations_enabled: true,
            labels_enabled: true,
            overdrive_enabled: false, // Off by default per requirements
            emoji_width_offset: 0,    // Will be set from detection at startup
        }
    }
}

/// Configuration for latency ring thresholds
#[derive(Debug, Clone)]
pub struct LatencyConfig {
    /// Threshold for "low latency" bucket in milliseconds
    pub low_threshold_ms: u64,

    /// Threshold for "high latency" bucket in milliseconds
    pub high_threshold_ms: u64,
}

impl Default for LatencyConfig {
    fn default() -> Self {
        Self {
            low_threshold_ms: 50,
            high_threshold_ms: 200,
        }
    }
}

/// Configuration for refresh intervals (unified)
#[derive(Debug, Clone)]
pub struct RefreshConfig {
    /// Refresh interval in milliseconds (50-1000ms)
    /// Data collection uses this * DATA_REFRESH_MULTIPLIER
    pub refresh_ms: u64,

    /// Timestamp of last interval change (for visual feedback)
    pub last_change: Option<Instant>,
}

impl RefreshConfig {
    /// Create a new RefreshConfig with default values
    pub fn new() -> Self {
        Self {
            refresh_ms: 500,
            last_change: None,
        }
    }

    /// Get UI refresh interval as Duration
    pub fn ui_interval(&self) -> Duration {
        Duration::from_millis(self.refresh_ms)
    }

    /// Get data refresh interval as Duration (10x UI interval)
    pub fn data_interval(&self) -> Duration {
        Duration::from_millis(self.refresh_ms * DATA_REFRESH_MULTIPLIER)
    }
}

impl Default for RefreshConfig {
    fn default() -> Self {
        Self::new()
    }
}
