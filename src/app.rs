// Application state management

use crate::net::{self, Connection};
use std::time::{Duration, Instant};

/// Number of log entries in the grimoire (for bounds checking)
const LOG_ENTRY_COUNT: usize = 6;

/// Tick interval for pulse animation (100ms)
const TICK_INTERVAL_MS: u128 = 100;

/// Blink interval for zombie animation (500ms)
const BLINK_INTERVAL_MS: u128 = 500;

/// Connection refresh interval (1 second)
const CONN_REFRESH_INTERVAL: Duration = Duration::from_secs(1);

/// Main application state
pub struct AppState {
    /// Whether the application is running
    pub running: bool,

    /// Currently selected node index in the network map
    pub selected_node: usize,

    /// Currently selected log entry index
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

        // Refresh connections every 1 second
        let elapsed_conn = now.duration_since(self.last_conn_refresh);
        if elapsed_conn >= CONN_REFRESH_INTERVAL {
            self.refresh_connections();
        }
    }

    /// Refresh network connections from /proc/net/tcp
    /// Read-only operation following security-domain guidelines
    pub fn refresh_connections(&mut self) {
        self.last_conn_refresh = Instant::now();

        match net::collect_connections() {
            Ok(conns) => {
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
        let new_value = (base_value + variation).max(10.0).min(100.0) as u64;

        // Add to history
        self.traffic_history.push(new_value);
    }

    /// Move log selection up (decrease index)
    pub fn select_previous_log(&mut self) {
        if self.selected_log > 0 {
            self.selected_log -= 1;
        }
    }

    /// Move log selection down (increase index)
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
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
