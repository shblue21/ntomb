// UI rendering module
//
// This module contains all UI rendering components for ntomb.
// The main draw() function orchestrates rendering of all UI panels.

mod banner;
mod graveyard;
mod grimoire;
mod inspector;
mod status_bar;

// Re-export graveyard types for external use (may be used by tests or future modules)
#[allow(unused_imports)]
pub use graveyard::{
    calculate_endpoint_position, classify_endpoint, classify_latency, draw_coffin_block,
    draw_latency_rings, has_latency_data, is_heavy_talker, particle_position, EndpointNode,
    EndpointType,
};

use crate::app::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use banner::render_banner;
use graveyard::render_network_map;
use grimoire::render_grimoire;
use inspector::render_soul_inspector;
use status_bar::render_status_bar;

/// Main UI drawing function
pub fn draw(f: &mut Frame, app: &mut AppState) {
    let size = f.area();

    // Main layout: banner, body, status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Banner
            Constraint::Min(0),     // Body
            Constraint::Length(3),  // Status bar
        ])
        .split(size);

    // Banner
    render_banner(f, chunks[0], app);

    // Body: Network map + right panels
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(65), // Network map
            Constraint::Percentage(35), // Right panels
        ])
        .split(chunks[1]);

    render_network_map(f, body_chunks[0], app);
    
    // Right side: Soul Inspector + Grimoire
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Soul Inspector
            Constraint::Percentage(40), // Grimoire
        ])
        .split(body_chunks[1]);
    
    render_soul_inspector(f, right_chunks[0], app);
    render_grimoire(f, right_chunks[1], app);

    // Status bar
    render_status_bar(f, chunks[2], app);
}
