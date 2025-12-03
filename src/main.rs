// ntomb - Network Tomb: Process-centric network visualization
// A Halloween-themed TUI for the Kiroween hackathon

mod app;
mod ui;

use anyhow::Result;
use app::AppState;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let res = run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut app = AppState::new();
    let tick_rate = std::time::Duration::from_millis(100);

    loop {
        // Draw the UI
        terminal.draw(|f| ui::draw(f, &app))?;

        // Check if app should exit
        if !app.running {
            return Ok(());
        }

        // Handle events with timeout
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    // Quit on 'q', 'Q', or Esc
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                        app.running = false;
                    }
                    // Navigate logs with arrow keys
                    KeyCode::Up => {
                        app.select_previous_log();
                    }
                    KeyCode::Down => {
                        app.select_next_log();
                    }
                    // Switch panel with Tab (placeholder for now)
                    KeyCode::Tab => {
                        app.switch_panel();
                    }
                    _ => {}
                }
            }
        }
    }
}


