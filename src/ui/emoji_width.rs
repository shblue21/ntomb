// Emoji width detection module
//
// Detects the actual rendered width of emoji characters in the terminal
// by querying cursor position before and after printing test characters.
//
// This solves the cross-platform issue where unicode-width crate returns
// a width value that doesn't match the actual terminal rendering.
// - macOS terminals typically render emoji as 2 cells
// - Linux terminals vary: some render as 1 cell, some as 2

use crossterm::{
    cursor,
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Write};
use std::sync::OnceLock;

/// Cached emoji width offset detected at startup
/// Positive value means emoji renders wider than unicode-width reports
/// Negative value means emoji renders narrower than unicode-width reports
static EMOJI_WIDTH_OFFSET: OnceLock<i32> = OnceLock::new();

/// Test emoji characters used for width detection
/// Using common emoji that appear in ntomb UI
const TEST_EMOJIS: &[&str] = &["🎃", "⚰️", "👻", "💀", "👑"];

/// Expected width from unicode-width crate for test emoji
/// Most emoji are reported as width 2 by unicode-width
const EXPECTED_UNICODE_WIDTH: i32 = 2;

/// Configuration for emoji width handling
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmojiWidthConfig {
    /// Detected offset: actual_width - unicode_width_reported
    /// 0 = no correction needed (macOS typical)
    /// -1 = emoji renders 1 cell narrower than reported (some Linux terminals)
    pub offset: i32,
    
    /// Whether detection was successful
    pub detected: bool,
    
    /// Fallback mode: use ASCII instead of emoji
    pub use_ascii_fallback: bool,
}

impl Default for EmojiWidthConfig {
    fn default() -> Self {
        Self {
            offset: 0,
            detected: false,
            use_ascii_fallback: false,
        }
    }
}

impl EmojiWidthConfig {
    /// Calculate the actual display width of a string containing emoji
    /// 
    /// Applies the detected offset correction to unicode-width's calculation.
    /// 
    /// # Arguments
    /// * `s` - The string to measure
    /// 
    /// # Returns
    /// Corrected width in terminal cells
    pub fn corrected_width(&self, s: &str) -> usize {
        use unicode_width::UnicodeWidthStr;
        
        let base_width = s.width() as i32;
        
        // Count emoji characters that need correction
        let emoji_count = count_emoji_chars(s) as i32;
        
        // Apply offset correction for each emoji
        let corrected = base_width + (emoji_count * self.offset);
        
        corrected.max(0) as usize
    }
    
    /// Get the width offset to apply when positioning emoji
    /// 
    /// Returns half the offset for centering calculations
    #[allow(dead_code)]
    pub fn centering_offset(&self) -> f64 {
        (self.offset as f64) / 2.0
    }
}

/// Count the number of emoji characters in a string
/// 
/// Counts characters that are likely to have width rendering issues:
/// - Characters with emoji presentation selectors
/// - Characters in emoji ranges
fn count_emoji_chars(s: &str) -> usize {
    s.chars().filter(|c| is_emoji_char(*c)).count()
}

/// Check if a character is an emoji that may have width issues
fn is_emoji_char(c: char) -> bool {
    let code = c as u32;
    
    // Common emoji ranges that have width issues
    matches!(code,
        // Miscellaneous Symbols and Pictographs
        0x1F300..=0x1F5FF |
        // Emoticons
        0x1F600..=0x1F64F |
        // Transport and Map Symbols
        0x1F680..=0x1F6FF |
        // Supplemental Symbols and Pictographs
        0x1F900..=0x1F9FF |
        // Symbols and Pictographs Extended-A
        0x1FA00..=0x1FA6F |
        // Symbols and Pictographs Extended-B
        0x1FA70..=0x1FAFF |
        // Dingbats
        0x2700..=0x27BF |
        // Miscellaneous Symbols
        0x2600..=0x26FF |
        // Box Drawing (coffin characters)
        0x2500..=0x257F |
        // Variation Selectors (emoji presentation)
        0xFE00..=0xFE0F
    )
}

/// Detect emoji width by querying terminal cursor position
/// 
/// This function:
/// 1. Saves cursor position
/// 2. Prints a test emoji
/// 3. Queries new cursor position
/// 4. Calculates actual rendered width
/// 5. Compares with unicode-width's reported width
/// 
/// # Returns
/// EmojiWidthConfig with detected offset, or default if detection fails
pub fn detect_emoji_width() -> EmojiWidthConfig {
    // Check environment variable override first
    if let Ok(env_offset) = std::env::var("NTOMB_EMOJI_WIDTH_OFFSET") {
        if let Ok(offset) = env_offset.parse::<i32>() {
            return EmojiWidthConfig {
                offset,
                detected: true,
                use_ascii_fallback: false,
            };
        }
    }
    
    // Check for ASCII fallback mode
    if std::env::var("NTOMB_ASCII_MODE").is_ok() || std::env::var("NO_COLOR").is_ok() {
        return EmojiWidthConfig {
            offset: 0,
            detected: true,
            use_ascii_fallback: true,
        };
    }
    
    // Try to detect actual emoji width
    match detect_emoji_width_internal() {
        Ok(offset) => EmojiWidthConfig {
            offset,
            detected: true,
            use_ascii_fallback: false,
        },
        Err(_) => {
            // Detection failed, use platform-specific defaults
            EmojiWidthConfig {
                offset: get_platform_default_offset(),
                detected: false,
                use_ascii_fallback: false,
            }
        }
    }
}

/// Get platform-specific default offset when detection fails
fn get_platform_default_offset() -> i32 {
    #[cfg(target_os = "macos")]
    {
        0 // macOS terminals typically render emoji correctly
    }
    
    #[cfg(target_os = "linux")]
    {
        // Linux terminals often render emoji as 1 cell instead of 2
        // This is a conservative default - actual detection is preferred
        -1
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        0
    }
}

/// Internal function to detect emoji width using cursor position query
fn detect_emoji_width_internal() -> io::Result<i32> {
    let mut stdout = io::stdout();
    
    // We need raw mode for cursor position query
    let was_raw = terminal::is_raw_mode_enabled().unwrap_or(false);
    
    if !was_raw {
        enable_raw_mode()?;
    }
    
    let result = (|| -> io::Result<i32> {
        // Save cursor position
        execute!(stdout, cursor::SavePosition)?;
        
        // Move to a known position (column 0)
        execute!(stdout, cursor::MoveToColumn(0))?;
        stdout.flush()?;
        
        // Get starting position
        let (start_col, _) = cursor::position()?;
        
        // Print test emoji
        let test_emoji = TEST_EMOJIS[0]; // 🎃
        print!("{}", test_emoji);
        stdout.flush()?;
        
        // Small delay to ensure terminal processes the output
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // Get ending position
        let (end_col, _) = cursor::position()?;
        
        // Restore cursor position
        execute!(stdout, cursor::RestorePosition)?;
        
        // Clear the test emoji by overwriting with spaces
        execute!(stdout, cursor::MoveToColumn(0))?;
        print!("    "); // Clear any residue
        execute!(stdout, cursor::RestorePosition)?;
        stdout.flush()?;
        
        // Calculate actual width
        let actual_width = (end_col - start_col) as i32;
        
        // Calculate offset: actual - expected
        let offset = actual_width - EXPECTED_UNICODE_WIDTH;
        
        Ok(offset)
    })();
    
    // Restore raw mode state
    if !was_raw {
        let _ = disable_raw_mode();
    }
    
    result
}

/// Initialize emoji width detection (call once at startup)
/// 
/// This should be called before entering the main TUI loop.
/// The result is cached for the lifetime of the application.
/// 
/// # Returns
/// Reference to the cached EmojiWidthConfig
pub fn init_emoji_width_detection() -> &'static EmojiWidthConfig {
    EMOJI_WIDTH_OFFSET.get_or_init(|| {
        let config = detect_emoji_width();
        
        // Log the detection result for debugging
        if config.detected {
            tracing::info!(
                offset = config.offset,
                ascii_fallback = config.use_ascii_fallback,
                "Emoji width detection completed"
            );
        } else {
            tracing::warn!(
                offset = config.offset,
                "Emoji width detection failed, using platform default"
            );
        }
        
        config.offset
    });
    
    // Return a static config based on the cached offset
    static CONFIG: OnceLock<EmojiWidthConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let offset = *EMOJI_WIDTH_OFFSET.get().unwrap_or(&0);
        EmojiWidthConfig {
            offset,
            detected: true,
            use_ascii_fallback: std::env::var("NTOMB_ASCII_MODE").is_ok(),
        }
    })
}

/// Get the current emoji width configuration
/// 
/// Returns the cached configuration if available, otherwise detects it.
#[allow(dead_code)]
pub fn get_emoji_width_config() -> &'static EmojiWidthConfig {
    init_emoji_width_detection()
}

/// Get the initially detected emoji width offset
/// 
/// This returns the offset detected at startup, which can be used
/// to initialize AppState's emoji_width_offset setting.
pub fn get_detected_offset() -> i32 {
    *EMOJI_WIDTH_OFFSET.get_or_init(|| {
        detect_emoji_width().offset
    })
}

/// Calculate corrected width for a string with custom offset
/// 
/// # Arguments
/// * `s` - The string to measure
/// * `offset` - Custom offset to apply (from AppState settings)
/// 
/// # Returns
/// Corrected width in terminal cells
pub fn corrected_str_width_with_offset(s: &str, offset: i32) -> usize {
    use unicode_width::UnicodeWidthStr;
    
    let base_width = s.width() as i32;
    let emoji_count = count_emoji_chars(s) as i32;
    let corrected = base_width + (emoji_count * offset);
    
    corrected.max(0) as usize
}

/// Calculate corrected width for a string
/// 
/// Convenience function that uses the cached emoji width configuration.
/// 
/// # Arguments
/// * `s` - The string to measure
/// 
/// # Returns
/// Corrected width in terminal cells
#[allow(dead_code)]
pub fn corrected_str_width(s: &str) -> usize {
    get_emoji_width_config().corrected_width(s)
}

/// Get centering offset for emoji positioning with custom offset
/// 
/// # Arguments
/// * `offset` - Custom offset from AppState settings
/// 
/// # Returns
/// The offset to apply when centering emoji icons
pub fn emoji_centering_offset_with(offset: i32) -> f64 {
    (offset as f64) / 2.0
}

/// Get centering offset for emoji positioning
/// 
/// Returns the offset to apply when centering emoji icons.
#[allow(dead_code)]
pub fn emoji_centering_offset() -> f64 {
    get_emoji_width_config().centering_offset()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_count_emoji_chars() {
        assert_eq!(count_emoji_chars("hello"), 0);
        assert_eq!(count_emoji_chars("🎃"), 1);
        assert_eq!(count_emoji_chars("🎃👻💀"), 3);
        assert_eq!(count_emoji_chars("hello 🎃 world"), 1);
    }
    
    #[test]
    fn test_is_emoji_char() {
        assert!(is_emoji_char('🎃'));
        assert!(is_emoji_char('👻'));
        assert!(is_emoji_char('💀'));
        assert!(!is_emoji_char('a'));
        assert!(!is_emoji_char('1'));
    }
    
    #[test]
    fn test_emoji_width_config_corrected_width() {
        let config = EmojiWidthConfig {
            offset: -1,
            detected: true,
            use_ascii_fallback: false,
        };
        
        // "🎃" has unicode-width of 2, with offset -1, corrected = 1
        let width = config.corrected_width("🎃");
        assert_eq!(width, 1);
        
        // "hello" has no emoji, width stays the same
        let width = config.corrected_width("hello");
        assert_eq!(width, 5);
    }
    
    #[test]
    fn test_default_config() {
        let config = EmojiWidthConfig::default();
        assert_eq!(config.offset, 0);
        assert!(!config.detected);
        assert!(!config.use_ascii_fallback);
    }
}
