// Application state management

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
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
