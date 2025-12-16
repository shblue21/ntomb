// ntomb - Network Tomb: Process-centric network visualization
// A Halloween-themed TUI for the Kiroween hackathon

mod app;
mod net;
mod procfs;
mod theme;
mod ui;

use anyhow::Result;
use app::{event::handle_key_event, AppState};
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> Result<()> {
    // Detect emoji width before entering alternate screen
    // This queries cursor position which requires the main terminal
    let _emoji_config = ui::emoji_width::init_emoji_width_detection();
    
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
        app.on_tick();
        app.update_frame_time();
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if !app.running {
            return Ok(());
        }

        if event::poll(app.refresh_config.ui_interval())? {
            if let Event::Key(key) = event::read()? {
                handle_key_event(&mut app, key.code);
            }
        }
    }
}
