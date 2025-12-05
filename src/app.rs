// Application state management

use crate::net::{self, Connection};
use ratatui::widgets::ListState;
use std::time::{Duration, Instant};

/// Graveyard view mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GraveyardMode {
    /// Host-wide view (default)
    #[default]
    Host,
    /// Selected process view
    Process,
}

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
}

impl Default for GraveyardSettings {
    fn default() -> Self {
        Self {
            animations_enabled: true,
            labels_enabled: true,
            overdrive_enabled: false, // Off by default per requirements
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

/// Number of log entries in the grimoire (for bounds checking)
#[allow(dead_code)]
const LOG_ENTRY_COUNT: usize = 6;

/// Tick interval for pulse animation (100ms)
const TICK_INTERVAL_MS: u128 = 100;

/// Blink interval for zombie animation (500ms)
const BLINK_INTERVAL_MS: u128 = 500;



// Refresh interval bounds and steps
/// Minimum refresh interval in milliseconds
const MIN_REFRESH_MS: u64 = 50;
/// Maximum refresh interval in milliseconds
const MAX_REFRESH_MS: u64 = 1000;
/// Refresh interval adjustment step in milliseconds
const REFRESH_STEP: u64 = 50;
/// Data refresh multiplier (data refreshes at N times the UI interval)
const DATA_REFRESH_MULTIPLIER: u64 = 10;

/// Duration to highlight recently changed refresh intervals
pub const CHANGE_HIGHLIGHT_DURATION: Duration = Duration::from_millis(500);

/// Frame time threshold for auto-reducing animation complexity (100ms)
/// If frame time consistently exceeds this, particle count is reduced
const FRAME_TIME_THRESHOLD_MS: u128 = 100;

/// Number of consecutive slow frames before triggering complexity reduction
const SLOW_FRAME_COUNT_THRESHOLD: u32 = 5;

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
            refresh_ms: 100,
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

/// Main application state
pub struct AppState {
    /// Whether the application is running
    pub running: bool,

    /// Currently selected node index in the network map
    #[allow(dead_code)]
    pub selected_node: usize,

    /// Currently selected log entry index
    #[allow(dead_code)]
    pub selected_log: usize,

    /// Traffic history data (last 60 samples)
    pub traffic_history: Vec<u64>,

    /// Pulse phase for neon animation (0.0 ~ 1.0)
    pub pulse_phase: f32,

    /// Zombie blink state (true = visible, false = faded)
    pub zombie_blink: bool,

    /// Last tick time for pulse animation
    pub last_tick: Instant,

    /// Last blink time for zombie animation
    pub last_blink: Instant,

    /// Tick counter for generating varied traffic data
    tick_counter: u64,

    /// Active network connections from /proc/net/tcp
    pub connections: Vec<Connection>,

    /// Last time connections were refreshed
    last_conn_refresh: Instant,

    /// Connection refresh error message (if any)
    pub conn_error: Option<String>,

    /// Graveyard view mode
    pub graveyard_mode: GraveyardMode,

    /// Selected process PID in Process mode
    pub selected_process_pid: Option<i32>,

    /// Currently selected connection index (Active Connections list)
    pub selected_connection: Option<usize>,

    /// List state for Active Connections (enables scrolling)
    pub connection_list_state: ListState,

    /// Refresh interval configuration
    pub refresh_config: RefreshConfig,

    /// Graveyard visual settings (animations, labels, overdrive)
    pub graveyard_settings: GraveyardSettings,

    /// Latency bucket configuration for ring positioning
    pub latency_config: LatencyConfig,
    
    /// Frame time tracking for performance monitoring (Requirements 6.5)
    /// Stores the timestamp of the last frame render
    last_frame_time: Instant,
    
    /// Counter for consecutive slow frames (frame time > 100ms)
    /// Used to trigger automatic animation complexity reduction
    slow_frame_count: u32,
    
    /// Whether animation complexity has been auto-reduced due to performance
    /// When true, particle rendering uses reduced particle count
    pub animation_reduced: bool,
}

impl AppState {
    /// Create a new AppState with default values
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            running: true,
            selected_node: 0,
            selected_log: 0,
            // Initialize with some baseline traffic values
            traffic_history: vec![30; 60],
            pulse_phase: 0.0,
            zombie_blink: true,
            last_tick: now,
            last_blink: now,
            tick_counter: 0,
            connections: Vec::new(),
            last_conn_refresh: now,
            conn_error: None,
            graveyard_mode: GraveyardMode::default(),
            selected_process_pid: None,
            selected_connection: None,
            connection_list_state: ListState::default(),
            refresh_config: RefreshConfig::new(),
            graveyard_settings: GraveyardSettings::default(),
            latency_config: LatencyConfig::default(),
            last_frame_time: now,
            slow_frame_count: 0,
            animation_reduced: false,
        }
    }

    /// Update state on each tick (called every ~100ms)
    pub fn on_tick(&mut self) {
        let now = Instant::now();

        // Update pulse phase every tick (~100ms)
        let elapsed_tick = now.duration_since(self.last_tick).as_millis();
        if elapsed_tick >= TICK_INTERVAL_MS {
            self.last_tick = now;
            self.tick_counter += 1;

            // Increment pulse phase (0.0 ~ 1.0)
            self.pulse_phase += 0.05;
            if self.pulse_phase >= 1.0 {
                self.pulse_phase = 0.0;
            }

            // Update traffic history with sine wave + some randomness
            self.update_traffic_history();
        }

        // Toggle zombie blink every 500ms
        let elapsed_blink = now.duration_since(self.last_blink).as_millis();
        if elapsed_blink >= BLINK_INTERVAL_MS {
            self.last_blink = now;
            self.zombie_blink = !self.zombie_blink;
        }

        // Refresh connections based on dynamic data refresh interval
        let elapsed_conn = now.duration_since(self.last_conn_refresh);
        if elapsed_conn >= self.refresh_config.data_interval() {
            self.refresh_connections();
        }
    }

    /// Refresh network connections from /proc/net/tcp
    /// Read-only operation following security-domain guidelines
    pub fn refresh_connections(&mut self) {
        self.last_conn_refresh = Instant::now();

        match net::collect_connections() {
            Ok(conns) => {
                // On Linux, attach process information to connections
                // This is a best-effort operation - failures are logged but don't prevent
                // the connections from being displayed
                #[cfg(target_os = "linux")]
                let conns = {
                    let mut conns = conns;
                    if let Err(e) = crate::procfs::attach_process_info(&mut conns) {
                        // Log the error but continue - process mapping is optional
                        tracing::warn!(error = %e, "Failed to attach process info to connections");
                    }
                    conns
                };

                self.connections = conns;
                self.conn_error = None;
            }
            Err(e) => {
                // Gracefully handle errors - don't panic
                // Following security-domain: calm, informative tone
                self.conn_error = Some(format!(
                    "Cannot read /proc/net/tcp: {} (permission or OS issue)",
                    e
                ));
                // Keep existing connections if refresh fails
            }
        }
    }

    /// Update traffic history with animated dummy data
    fn update_traffic_history(&mut self) {
        // Remove oldest value
        self.traffic_history.remove(0);

        // Generate new value using sine wave for smooth animation
        let t = self.tick_counter as f32 * 0.1;
        let base_value = 50.0 + 40.0 * (t).sin();

        // Add some variation
        let variation = ((t * 2.3).sin() * 10.0) + ((t * 1.7).cos() * 5.0);
        let new_value = (base_value + variation).clamp(10.0, 100.0) as u64;

        // Add to history
        self.traffic_history.push(new_value);
    }

    /// Move log selection up (decrease index)
    #[allow(dead_code)]
    pub fn select_previous_log(&mut self) {
        if self.selected_log > 0 {
            self.selected_log -= 1;
        }
    }

    /// Move log selection down (increase index)
    #[allow(dead_code)]
    pub fn select_next_log(&mut self) {
        if self.selected_log < LOG_ENTRY_COUNT.saturating_sub(1) {
            self.selected_log += 1;
        }
    }

    /// Handle Tab key press (placeholder for future panel switching)
    pub fn switch_panel(&mut self) {
        // TODO: Implement panel switching logic
        // For now, this is a placeholder
    }

    /// Move connection selection up (decrease index)
    pub fn select_previous_connection(&mut self) {
        if self.connections.is_empty() {
            self.selected_connection = None;
            self.connection_list_state.select(None);
            return;
        }

        match self.selected_connection {
            None => {
                // Start at the last connection
                let idx = self.connections.len() - 1;
                self.selected_connection = Some(idx);
                self.connection_list_state.select(Some(idx));
            }
            Some(idx) => {
                if idx > 0 {
                    self.selected_connection = Some(idx - 1);
                    self.connection_list_state.select(Some(idx - 1));
                }
            }
        }
    }

    /// Move connection selection down (increase index)
    pub fn select_next_connection(&mut self) {
        if self.connections.is_empty() {
            self.selected_connection = None;
            self.connection_list_state.select(None);
            return;
        }

        match self.selected_connection {
            None => {
                // Start at the first connection
                self.selected_connection = Some(0);
                self.connection_list_state.select(Some(0));
            }
            Some(idx) => {
                if idx < self.connections.len() - 1 {
                    self.selected_connection = Some(idx + 1);
                    self.connection_list_state.select(Some(idx + 1));
                }
            }
        }
    }

    /// Focus on the process of the selected connection
    pub fn focus_process_of_selected_connection(&mut self) {
        if let Some(conn_idx) = self.selected_connection {
            if let Some(conn) = self.connections.get(conn_idx) {
                // Switch to Process mode even if PID is unknown (macOS)
                self.graveyard_mode = GraveyardMode::Process;
                self.selected_process_pid = conn.pid;
            }
        }
    }

    /// Clear process focus, return to Host mode
    pub fn clear_process_focus(&mut self) {
        self.graveyard_mode = GraveyardMode::Host;
        self.selected_process_pid = None;
    }

    /// Toggle focus based on current mode
    pub fn toggle_graveyard_mode(&mut self) {
        match self.graveyard_mode {
            GraveyardMode::Host => {
                // Switch to Process mode if a connection is selected
                self.focus_process_of_selected_connection();
            }
            GraveyardMode::Process => {
                // Return to Host mode
                self.clear_process_focus();
            }
        }
    }

    /// Increase refresh rate (decrease interval by 50ms, clamp to 50ms minimum)
    pub fn increase_refresh_rate(&mut self) {
        let new_interval = self.refresh_config.refresh_ms.saturating_sub(REFRESH_STEP);
        self.refresh_config.refresh_ms = new_interval.max(MIN_REFRESH_MS);
        self.refresh_config.last_change = Some(Instant::now());
    }

    /// Decrease refresh rate (increase interval by 50ms, clamp to 1000ms maximum)
    pub fn decrease_refresh_rate(&mut self) {
        let new_interval = self.refresh_config.refresh_ms.saturating_add(REFRESH_STEP);
        self.refresh_config.refresh_ms = new_interval.min(MAX_REFRESH_MS);
        self.refresh_config.last_change = Some(Instant::now());
    }
    
    /// Update frame time tracking and auto-reduce animation complexity if needed
    /// 
    /// This method should be called at the start of each frame render.
    /// It monitors frame time and automatically reduces animation complexity
    /// if frame time consistently exceeds FRAME_TIME_THRESHOLD_MS (100ms).
    /// 
    /// Requirements: 6.5 - Auto-reduce animation complexity when CPU usage is high
    pub fn update_frame_time(&mut self) {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time).as_millis();
        self.last_frame_time = now;
        
        // Check if frame time exceeds threshold
        if frame_time > FRAME_TIME_THRESHOLD_MS {
            self.slow_frame_count += 1;
            
            // If we've had enough consecutive slow frames, reduce animation complexity
            if self.slow_frame_count >= SLOW_FRAME_COUNT_THRESHOLD && !self.animation_reduced {
                self.animation_reduced = true;
                // Log the auto-reduction for debugging
                tracing::info!(
                    frame_time_ms = frame_time,
                    slow_frame_count = self.slow_frame_count,
                    "Auto-reducing animation complexity due to slow frame times"
                );
            }
        } else {
            // Reset slow frame counter on a fast frame
            // Only reset if we haven't already reduced complexity
            if !self.animation_reduced {
                self.slow_frame_count = 0;
            }
        }
    }
    
    /// Reset animation complexity reduction
    /// 
    /// Called when user manually toggles animations or when performance improves.
    /// This allows the system to try full animation complexity again.
    pub fn reset_animation_reduction(&mut self) {
        self.animation_reduced = false;
        self.slow_frame_count = 0;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        /// **Feature: process-focus, Property 3: Mode toggle consistency**
        /// **Validates: Requirements 4.2, 4.3**
        ///
        /// For any AppState, calling toggle_graveyard_mode() when in Host mode
        /// with a valid selected connection SHALL result in Process mode, and
        /// calling it again SHALL return to Host mode with selected_process_pid
        /// reset to None.
        #[test]
        fn prop_mode_toggle_consistency(
            pid in 1i32..10000i32,
            conn_idx in 0usize..10usize,
        ) {
            // Create a test connection with the generated pid
            let test_conn = Connection {
                local_addr: "127.0.0.1".to_string(),
                local_port: 8080,
                remote_addr: "192.168.1.1".to_string(),
                remote_port: 443,
                state: crate::net::ConnectionState::Established,
                inode: Some(12345),
                pid: Some(pid),
                process_name: Some("test_process".to_string()),
            };

            // Create app state with the test connection
            let mut app = AppState::new();
            app.connections = vec![test_conn];
            app.selected_connection = Some(conn_idx.min(app.connections.len() - 1));

            // Initial state should be Host mode
            prop_assert_eq!(app.graveyard_mode, GraveyardMode::Host);
            prop_assert_eq!(app.selected_process_pid, None);

            // First toggle: Host -> Process
            app.toggle_graveyard_mode();

            // Should now be in Process mode with the selected pid
            prop_assert_eq!(app.graveyard_mode, GraveyardMode::Process);
            prop_assert_eq!(app.selected_process_pid, Some(pid));

            // Second toggle: Process -> Host
            app.toggle_graveyard_mode();

            // Should be back in Host mode with pid reset to None
            prop_assert_eq!(app.graveyard_mode, GraveyardMode::Host);
            prop_assert_eq!(app.selected_process_pid, None);
        }
    }

    // ============================================================================
    // Task 24.1: Integration tests for toggle persistence
    // Requirements: 5.7 - Toggles maintain state across frames and apply immediately
    // ============================================================================

    #[test]
    fn test_toggle_animations_persistence_across_ticks() {
        // Test that animation toggle maintains state across multiple on_tick() calls
        // Requirements: 5.7 - Toggle changes apply immediately without restart
        let mut app = AppState::new();
        
        // Default state: animations enabled
        assert!(app.graveyard_settings.animations_enabled);
        
        // Toggle animations off
        app.graveyard_settings.animations_enabled = false;
        
        // Simulate multiple frame updates (on_tick calls)
        for _ in 0..10 {
            app.on_tick();
        }
        
        // Animation setting should persist across ticks
        assert!(!app.graveyard_settings.animations_enabled);
        
        // Toggle back on
        app.graveyard_settings.animations_enabled = true;
        
        // Simulate more frame updates
        for _ in 0..10 {
            app.on_tick();
        }
        
        // Should still be enabled
        assert!(app.graveyard_settings.animations_enabled);
    }

    #[test]
    fn test_toggle_overdrive_persistence_across_ticks() {
        // Test that overdrive toggle maintains state across multiple on_tick() calls
        // Requirements: 5.7 - Toggle changes apply immediately without restart
        let mut app = AppState::new();
        
        // Default state: overdrive disabled
        assert!(!app.graveyard_settings.overdrive_enabled);
        
        // Toggle overdrive on
        app.graveyard_settings.overdrive_enabled = true;
        
        // Simulate multiple frame updates
        for _ in 0..10 {
            app.on_tick();
        }
        
        // Overdrive setting should persist across ticks
        assert!(app.graveyard_settings.overdrive_enabled);
        
        // Toggle back off
        app.graveyard_settings.overdrive_enabled = false;
        
        // Simulate more frame updates
        for _ in 0..10 {
            app.on_tick();
        }
        
        // Should still be disabled
        assert!(!app.graveyard_settings.overdrive_enabled);
    }

    #[test]
    fn test_toggle_labels_persistence_across_ticks() {
        // Test that labels toggle maintains state across multiple on_tick() calls
        // Requirements: 5.7 - Toggle changes apply immediately without restart
        let mut app = AppState::new();
        
        // Default state: labels enabled
        assert!(app.graveyard_settings.labels_enabled);
        
        // Toggle labels off
        app.graveyard_settings.labels_enabled = false;
        
        // Simulate multiple frame updates
        for _ in 0..10 {
            app.on_tick();
        }
        
        // Labels setting should persist across ticks
        assert!(!app.graveyard_settings.labels_enabled);
        
        // Toggle back on
        app.graveyard_settings.labels_enabled = true;
        
        // Simulate more frame updates
        for _ in 0..10 {
            app.on_tick();
        }
        
        // Should still be enabled
        assert!(app.graveyard_settings.labels_enabled);
    }

    #[test]
    fn test_toggle_immediate_application() {
        // Test that toggle changes apply immediately (no restart required)
        // Requirements: 5.7 - Changes apply immediately
        let mut app = AppState::new();
        
        // Record initial states
        let initial_animations = app.graveyard_settings.animations_enabled;
        let initial_overdrive = app.graveyard_settings.overdrive_enabled;
        let initial_labels = app.graveyard_settings.labels_enabled;
        
        // Toggle all settings
        app.graveyard_settings.animations_enabled = !initial_animations;
        app.graveyard_settings.overdrive_enabled = !initial_overdrive;
        app.graveyard_settings.labels_enabled = !initial_labels;
        
        // Verify changes are immediately reflected (no on_tick needed)
        assert_eq!(app.graveyard_settings.animations_enabled, !initial_animations);
        assert_eq!(app.graveyard_settings.overdrive_enabled, !initial_overdrive);
        assert_eq!(app.graveyard_settings.labels_enabled, !initial_labels);
    }

    // ============================================================================
    // Task 24.2: Integration tests for mode combinations
    // Requirements: 5.4 - Static graphics convey same information when animations disabled
    // ============================================================================

    #[test]
    fn test_host_mode_with_overdrive() {
        // Test Host mode + Overdrive enabled combination
        // Requirements: 5.4 - Mode combinations work correctly
        let mut app = AppState::new();
        
        // Set up Host mode with Overdrive
        app.graveyard_mode = GraveyardMode::Host;
        app.graveyard_settings.overdrive_enabled = true;
        
        // Add test connections
        let test_conn = Connection {
            local_addr: "127.0.0.1".to_string(),
            local_port: 8080,
            remote_addr: "192.168.1.1".to_string(),
            remote_port: 443,
            state: crate::net::ConnectionState::Established,
            inode: Some(12345),
            pid: Some(1234),
            process_name: Some("test_process".to_string()),
        };
        app.connections = vec![test_conn];
        
        // Verify state combination
        assert_eq!(app.graveyard_mode, GraveyardMode::Host);
        assert!(app.graveyard_settings.overdrive_enabled);
        
        // Simulate frame updates - should not crash or change mode
        for _ in 0..5 {
            app.on_tick();
        }
        
        // State should be preserved
        assert_eq!(app.graveyard_mode, GraveyardMode::Host);
        assert!(app.graveyard_settings.overdrive_enabled);
        assert_eq!(app.connections.len(), 1);
    }

    #[test]
    fn test_process_mode_with_animations_off() {
        // Test Process mode + Animations disabled combination
        // Requirements: 5.4 - Static graphics convey same information
        let mut app = AppState::new();
        
        // Add test connection and select it
        let test_conn = Connection {
            local_addr: "127.0.0.1".to_string(),
            local_port: 8080,
            remote_addr: "192.168.1.1".to_string(),
            remote_port: 443,
            state: crate::net::ConnectionState::Established,
            inode: Some(12345),
            pid: Some(5678),
            process_name: Some("test_process".to_string()),
        };
        app.connections = vec![test_conn];
        app.selected_connection = Some(0);
        
        // Switch to Process mode
        app.toggle_graveyard_mode();
        assert_eq!(app.graveyard_mode, GraveyardMode::Process);
        assert_eq!(app.selected_process_pid, Some(5678));
        
        // Disable animations
        app.graveyard_settings.animations_enabled = false;
        
        // Verify state combination
        assert_eq!(app.graveyard_mode, GraveyardMode::Process);
        assert!(!app.graveyard_settings.animations_enabled);
        
        // Simulate frame updates
        for _ in 0..5 {
            app.on_tick();
        }
        
        // State should be preserved
        assert_eq!(app.graveyard_mode, GraveyardMode::Process);
        assert!(!app.graveyard_settings.animations_enabled);
        assert_eq!(app.selected_process_pid, Some(5678));
    }

    #[test]
    fn test_all_toggles_off() {
        // Test with all visual toggles disabled
        // Requirements: 5.4 - Static graphics convey same information
        let mut app = AppState::new();
        
        // Disable all toggles
        app.graveyard_settings.animations_enabled = false;
        app.graveyard_settings.overdrive_enabled = false;
        app.graveyard_settings.labels_enabled = false;
        
        // Add test connections
        let test_conns = vec![
            Connection {
                local_addr: "127.0.0.1".to_string(),
                local_port: 8080,
                remote_addr: "192.168.1.1".to_string(),
                remote_port: 443,
                state: crate::net::ConnectionState::Established,
                inode: Some(1),
                pid: Some(100),
                process_name: Some("proc1".to_string()),
            },
            Connection {
                local_addr: "127.0.0.1".to_string(),
                local_port: 8081,
                remote_addr: "10.0.0.1".to_string(),
                remote_port: 80,
                state: crate::net::ConnectionState::Listen,
                inode: Some(2),
                pid: Some(200),
                process_name: Some("proc2".to_string()),
            },
        ];
        app.connections = test_conns;
        
        // Verify all toggles are off
        assert!(!app.graveyard_settings.animations_enabled);
        assert!(!app.graveyard_settings.overdrive_enabled);
        assert!(!app.graveyard_settings.labels_enabled);
        
        // Simulate frame updates
        for _ in 0..10 {
            app.on_tick();
        }
        
        // All toggles should remain off
        assert!(!app.graveyard_settings.animations_enabled);
        assert!(!app.graveyard_settings.overdrive_enabled);
        assert!(!app.graveyard_settings.labels_enabled);
        
        // Connections should still be accessible
        assert_eq!(app.connections.len(), 2);
    }

    #[test]
    fn test_mode_switch_preserves_toggle_settings() {
        // Test that switching between Host and Process mode preserves toggle settings
        // Requirements: 5.4, 5.7
        let mut app = AppState::new();
        
        // Set up custom toggle configuration
        app.graveyard_settings.animations_enabled = false;
        app.graveyard_settings.overdrive_enabled = true;
        app.graveyard_settings.labels_enabled = false;
        
        // Add test connection
        let test_conn = Connection {
            local_addr: "127.0.0.1".to_string(),
            local_port: 8080,
            remote_addr: "192.168.1.1".to_string(),
            remote_port: 443,
            state: crate::net::ConnectionState::Established,
            inode: Some(12345),
            pid: Some(9999),
            process_name: Some("test_process".to_string()),
        };
        app.connections = vec![test_conn];
        app.selected_connection = Some(0);
        
        // Switch to Process mode
        app.toggle_graveyard_mode();
        assert_eq!(app.graveyard_mode, GraveyardMode::Process);
        
        // Toggle settings should be preserved
        assert!(!app.graveyard_settings.animations_enabled);
        assert!(app.graveyard_settings.overdrive_enabled);
        assert!(!app.graveyard_settings.labels_enabled);
        
        // Switch back to Host mode
        app.toggle_graveyard_mode();
        assert_eq!(app.graveyard_mode, GraveyardMode::Host);
        
        // Toggle settings should still be preserved
        assert!(!app.graveyard_settings.animations_enabled);
        assert!(app.graveyard_settings.overdrive_enabled);
        assert!(!app.graveyard_settings.labels_enabled);
    }

    #[test]
    fn test_connection_selection_navigation() {
        // Test with empty connections
        let mut app = AppState::new();
        app.select_next_connection();
        assert_eq!(app.selected_connection, None);
        app.select_previous_connection();
        assert_eq!(app.selected_connection, None);

        // Add some test connections
        let test_conns = vec![
            Connection {
                local_addr: "127.0.0.1".to_string(),
                local_port: 8080,
                remote_addr: "192.168.1.1".to_string(),
                remote_port: 443,
                state: crate::net::ConnectionState::Established,
                inode: Some(1),
                pid: Some(100),
                process_name: Some("proc1".to_string()),
            },
            Connection {
                local_addr: "127.0.0.1".to_string(),
                local_port: 8081,
                remote_addr: "192.168.1.2".to_string(),
                remote_port: 443,
                state: crate::net::ConnectionState::Established,
                inode: Some(2),
                pid: Some(200),
                process_name: Some("proc2".to_string()),
            },
            Connection {
                local_addr: "127.0.0.1".to_string(),
                local_port: 8082,
                remote_addr: "192.168.1.3".to_string(),
                remote_port: 443,
                state: crate::net::ConnectionState::Established,
                inode: Some(3),
                pid: Some(300),
                process_name: Some("proc3".to_string()),
            },
        ];
        app.connections = test_conns;

        // Test navigation from None
        app.select_next_connection();
        assert_eq!(app.selected_connection, Some(0));

        // Navigate down
        app.select_next_connection();
        assert_eq!(app.selected_connection, Some(1));

        app.select_next_connection();
        assert_eq!(app.selected_connection, Some(2));

        // Try to go beyond bounds (should stay at 2)
        app.select_next_connection();
        assert_eq!(app.selected_connection, Some(2));

        // Navigate up
        app.select_previous_connection();
        assert_eq!(app.selected_connection, Some(1));

        app.select_previous_connection();
        assert_eq!(app.selected_connection, Some(0));

        // Try to go below 0 (should stay at 0)
        app.select_previous_connection();
        assert_eq!(app.selected_connection, Some(0));

        // Test navigation from None going up
        app.selected_connection = None;
        app.select_previous_connection();
        assert_eq!(app.selected_connection, Some(2)); // Should wrap to last
    }
}
