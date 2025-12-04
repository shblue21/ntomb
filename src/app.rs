// Application state management

use crate::net::{self, Connection};
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

    /// Refresh interval configuration
    pub refresh_config: RefreshConfig,
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
            refresh_config: RefreshConfig::new(),
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
            return;
        }

        match self.selected_connection {
            None => {
                // Start at the last connection
                self.selected_connection = Some(self.connections.len() - 1);
            }
            Some(idx) => {
                if idx > 0 {
                    self.selected_connection = Some(idx - 1);
                }
            }
        }
    }

    /// Move connection selection down (increase index)
    pub fn select_next_connection(&mut self) {
        if self.connections.is_empty() {
            self.selected_connection = None;
            return;
        }

        match self.selected_connection {
            None => {
                // Start at the first connection
                self.selected_connection = Some(0);
            }
            Some(idx) => {
                if idx < self.connections.len() - 1 {
                    self.selected_connection = Some(idx + 1);
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
