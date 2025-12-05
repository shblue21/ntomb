// Keyboard event handling
//
// This module contains the keyboard event handler that processes
// user input and updates the application state accordingly.

use super::AppState;
use crossterm::event::KeyCode;

/// Handle keyboard events and update application state
///
/// Returns `true` if the application should continue running,
/// `false` if it should exit.
///
/// # Arguments
/// * `app` - Mutable reference to the application state
/// * `key` - The key code that was pressed
///
/// # Key Bindings
/// - `q`, `Q`, `Esc` - Quit the application
/// - `Up` - Select previous connection
/// - `Down` - Select next connection
/// - `p`, `P` - Toggle graveyard mode (Host/Process)
/// - `Tab` - Switch panel (placeholder)
/// - `+`, `=` - Increase refresh rate
/// - `-`, `_` - Decrease refresh rate
/// - `a`, `A` - Toggle animations
/// - `h`, `H` - Toggle Kiroween Overdrive mode
/// - `t`, `T` - Toggle endpoint labels
pub fn handle_key_event(app: &mut AppState, key: KeyCode) -> bool {
    match key {
        // Quit on 'q', 'Q', or Esc
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            app.running = false;
            false
        }
        // Navigate connections with arrow keys
        KeyCode::Up => {
            app.select_previous_connection();
            true
        }
        KeyCode::Down => {
            app.select_next_connection();
            true
        }
        // Toggle graveyard mode with 'p' key
        KeyCode::Char('p') | KeyCode::Char('P') => {
            app.toggle_graveyard_mode();
            true
        }
        // Switch panel with Tab (placeholder for now)
        KeyCode::Tab => {
            app.switch_panel();
            true
        }
        // Refresh rate controls (unified)
        // + = slower refresh (increase interval)
        // - = faster refresh (decrease interval)
        KeyCode::Char('+') | KeyCode::Char('=') => {
            app.decrease_refresh_rate();
            true
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            app.increase_refresh_rate();
            true
        }
        // Toggle animations (Requirements 2.4, 5.1)
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.graveyard_settings.animations_enabled =
                !app.graveyard_settings.animations_enabled;
            // Reset animation reduction when user manually toggles animations
            // This allows the system to try full animation complexity again
            app.reset_animation_reduction();
            true
        }
        // Toggle Kiroween Overdrive mode (Requirements 4.1, 5.2)
        KeyCode::Char('h') | KeyCode::Char('H') => {
            app.graveyard_settings.overdrive_enabled =
                !app.graveyard_settings.overdrive_enabled;
            true
        }
        // Toggle endpoint labels (Requirements 3.6, 5.3)
        KeyCode::Char('t') | KeyCode::Char('T') => {
            app.graveyard_settings.labels_enabled =
                !app.graveyard_settings.labels_enabled;
            true
        }
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quit_keys() {
        let mut app = AppState::new();
        
        // Test 'q' key
        assert!(app.running);
        let result = handle_key_event(&mut app, KeyCode::Char('q'));
        assert!(!result);
        assert!(!app.running);
        
        // Reset and test 'Q' key
        app.running = true;
        let result = handle_key_event(&mut app, KeyCode::Char('Q'));
        assert!(!result);
        assert!(!app.running);
        
        // Reset and test Esc key
        app.running = true;
        let result = handle_key_event(&mut app, KeyCode::Esc);
        assert!(!result);
        assert!(!app.running);
    }

    #[test]
    fn test_toggle_animations() {
        let mut app = AppState::new();
        
        // Default: animations enabled
        assert!(app.graveyard_settings.animations_enabled);
        
        // Toggle off
        handle_key_event(&mut app, KeyCode::Char('a'));
        assert!(!app.graveyard_settings.animations_enabled);
        
        // Toggle on
        handle_key_event(&mut app, KeyCode::Char('A'));
        assert!(app.graveyard_settings.animations_enabled);
    }

    #[test]
    fn test_toggle_overdrive() {
        let mut app = AppState::new();
        
        // Default: overdrive disabled
        assert!(!app.graveyard_settings.overdrive_enabled);
        
        // Toggle on
        handle_key_event(&mut app, KeyCode::Char('h'));
        assert!(app.graveyard_settings.overdrive_enabled);
        
        // Toggle off
        handle_key_event(&mut app, KeyCode::Char('H'));
        assert!(!app.graveyard_settings.overdrive_enabled);
    }

    #[test]
    fn test_toggle_labels() {
        let mut app = AppState::new();
        
        // Default: labels enabled
        assert!(app.graveyard_settings.labels_enabled);
        
        // Toggle off
        handle_key_event(&mut app, KeyCode::Char('t'));
        assert!(!app.graveyard_settings.labels_enabled);
        
        // Toggle on
        handle_key_event(&mut app, KeyCode::Char('T'));
        assert!(app.graveyard_settings.labels_enabled);
    }

    #[test]
    fn test_refresh_rate_controls() {
        let mut app = AppState::new();
        let initial_rate = app.refresh_config.refresh_ms;
        
        // Increase rate (decrease interval)
        handle_key_event(&mut app, KeyCode::Char('+'));
        assert!(app.refresh_config.refresh_ms < initial_rate);
        
        // Decrease rate (increase interval)
        handle_key_event(&mut app, KeyCode::Char('-'));
        assert_eq!(app.refresh_config.refresh_ms, initial_rate);
    }
}
