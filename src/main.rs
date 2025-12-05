// ntomb - Network Tomb: Process-centric network visualization
// A Halloween-themed TUI for the Kiroween hackathon

mod app;
mod net;
mod procfs;
mod ui;

use anyhow::Result;
use app::AppState;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let res = run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut app = AppState::new();

    loop {
        // Update app state (animations, traffic history, etc.)
        app.on_tick();
        
        // Update frame time tracking for performance monitoring (Requirement 6.5)
        // This monitors frame time and auto-reduces animation complexity if needed
        app.update_frame_time();

        // Draw the UI
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // Check if app should exit
        if !app.running {
            return Ok(());
        }

        // Use dynamic UI refresh interval from config
        let tick_rate = app.refresh_config.ui_interval();

        // Handle events with timeout
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    // Quit on 'q', 'Q', or Esc
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                        app.running = false;
                    }
                    // Navigate connections with arrow keys
                    KeyCode::Up => {
                        app.select_previous_connection();
                    }
                    KeyCode::Down => {
                        app.select_next_connection();
                    }
                    // Toggle graveyard mode with 'p' key
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        app.toggle_graveyard_mode();
                    }
                    // Switch panel with Tab (placeholder for now)
                    KeyCode::Tab => {
                        app.switch_panel();
                    }
                    // Refresh rate controls (unified)
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        app.increase_refresh_rate();
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        app.decrease_refresh_rate();
                    }
                    // Toggle animations (Requirements 2.4, 5.1)
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        app.graveyard_settings.animations_enabled =
                            !app.graveyard_settings.animations_enabled;
                        // Reset animation reduction when user manually toggles animations
                        // This allows the system to try full animation complexity again
                        app.reset_animation_reduction();
                    }
                    // Toggle Kiroween Overdrive mode (Requirements 4.1, 5.2)
                    KeyCode::Char('h') | KeyCode::Char('H') => {
                        app.graveyard_settings.overdrive_enabled =
                            !app.graveyard_settings.overdrive_enabled;
                    }
                    // Toggle endpoint labels (Requirements 3.6, 5.3)
                    KeyCode::Char('t') | KeyCode::Char('T') => {
                        app.graveyard_settings.labels_enabled =
                            !app.graveyard_settings.labels_enabled;
                    }
                    _ => {}
                }
            }
        }
    }
}


