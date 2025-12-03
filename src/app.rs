// Application state management

/// Number of log entries in the grimoire (for bounds checking)
const LOG_ENTRY_COUNT: usize = 6;

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
}

impl AppState {
    /// Create a new AppState with default values
    pub fn new() -> Self {
        Self {
            running: true,
            selected_node: 0,
            selected_log: 0,
            // Initialize with 60 zero values for traffic history
            traffic_history: vec![0; 60],
        }
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
