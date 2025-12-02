---
inclusion: always
---

# ntomb Coding Style Guide

## Goals

This guide defines coding style and structural guidelines for ntomb to ensure:

- **Clean, testable code:** Code should be easy to understand, modify, and test in isolation
- **Clear separation of concerns:** Data collection, domain logic, and TUI rendering should be distinct layers
- **Consistent naming and error handling:** Future changes by Kiro or developers should maintain coherence across the codebase
- **Maintainability:** Code should be straightforward to debug, extend, and refactor

## Language & Dependencies

**Primary language:** Rust

**Dependency philosophy:**
- Prefer a small, focused dependency set
- Avoid heavyweight frameworks; use simple, well-maintained crates
- For TUI: Use Ratatui for rendering and Crossterm for terminal handling
- For CLI: Use clap for command-line interface and argument parsing
- For logging: Use tracing or env_logger for structured logging
- For error handling: Use thiserror for custom errors and anyhow for application errors
- Keep platform-specific code isolated in dedicated modules or behind feature flags

**Rationale:** A small binary with minimal dependencies is easier to audit, faster to compile, and more portable across Linux distributions. Rust's standard library and ecosystem provide excellent tools for system programming.

## Project Structure

Organize code into clear, focused modules following Rust conventions:

```
ntomb/
├── Cargo.toml               # Project manifest and dependencies
├── Cargo.lock               # Locked dependency versions
├── src/
│   ├── main.rs              # CLI entry point, argument parsing, orchestration
│   ├── lib.rs               # Library root, re-exports public API
│   ├── process/
│   │   ├── mod.rs           # Process module root
│   │   ├── scanner.rs       # Read /proc, find processes
│   │   └── types.rs         # Process data structures
│   ├── netscan/
│   │   ├── mod.rs           # NetScan module root
│   │   ├── scanner.rs       # Read /proc/net/*, parse connections
│   │   ├── parser.rs        # Parse /proc/net/tcp, /proc/net/udp
│   │   └── types.rs         # Connection, Protocol, State types
│   ├── graph/
│   │   ├── mod.rs           # Graph module root
│   │   ├── builder.rs       # Build graph from process + connections
│   │   ├── layout.rs        # Calculate node positions
│   │   └── types.rs         # GraphNode, GraphEdge, Graph
│   ├── tui/
│   │   ├── mod.rs           # TUI module root
│   │   ├── app.rs           # Application state and main loop
│   │   ├── widgets.rs       # Custom Ratatui widgets
│   │   └── event.rs         # Event handling
│   ├── theme/
│   │   ├── mod.rs           # Theme module root
│   │   ├── default.rs       # Default theme
│   │   └── halloween.rs     # Halloween theme
│   └── demo/
│       ├── mod.rs           # Demo module root
│       └── fixtures.rs      # Fake processes and connections
└── tests/
    └── integration/         # Integration tests
```

**Key principles:**
- **Use modules for organization:** Each major component gets its own module directory
- **TUI layer should only render:** Business logic and data transformation belong in `process`, `netscan`, and `graph` modules
- **Avoid mixing parsing with presentation:** The `netscan` module parses `/proc` data; the `tui` module displays it
- **Keep themes separate:** Theme logic should be isolated so switching themes is trivial
- **Demo mode is first-class:** Demo fixtures should be realistic and maintained alongside real code

## Naming & Style

**General naming:**
- Use clear, descriptive names: `build_process_graph`, `scan_tcp_connections`, `render_network_map`
- Avoid overly clever abstractions or cryptic abbreviations
- Prefer explicit over implicit: `parse_proc_net_tcp` is better than `parse_net`
- Use domain terminology consistently: "process", "connection", "endpoint", "node", "edge"

**Rust-specific conventions:**
- `snake_case` for functions, methods, variables, and modules: `find_by_pid`, `connection_count`
- `PascalCase` for types, traits, and enum variants: `Process`, `ConnectionState`, `NodeType`
- `SCREAMING_SNAKE_CASE` for constants: `MAX_NODES`, `DEFAULT_REFRESH_INTERVAL`
- Prefix boolean functions with `is_`, `has_`, `should_`: `is_suspicious`, `has_connections`, `should_group`
- Use `rustfmt` to automatically format code

**Function design:**
- Prefer small, focused functions (< 50 lines when possible)
- Each function should do one thing well
- Avoid deeply nested logic; extract helper functions
- Use descriptive parameter names
- Use `&self` for methods that don't need ownership, `&mut self` for mutations

**Error handling:**
- Return `Result<T, E>` for fallible operations
- Use `thiserror` for custom error types with meaningful messages
- Use `anyhow` for application-level error handling with context
- Use `?` operator for error propagation
- Provide context with `.context()` or `.with_context()`

**Example:**
```rust
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

// Good
pub fn find_process_by_pid(pid: u32) -> Result<Process> {
    let stat_path = format!("/proc/{}/stat", pid);
    let stat_content = fs::read_to_string(&stat_path)
        .with_context(|| format!("Failed to read process stat for PID {}", pid))?;
    parse_proc_stat(&stat_content)
}

// Avoid
fn find(p: u32) -> Process {
    let s = fs::read_to_string(format!("/proc/{}/stat", p)).unwrap();
    parse(s).unwrap()
}
```

## Error Handling & Logging

**User-facing errors:**
- Keep error messages short and actionable
- Include what went wrong and what the user can do
- Examples:
  - ✅ "Cannot read /proc/net/tcp: permission denied. Try running with sudo."
  - ✅ "Process 1234 not found. It may have exited."
  - ❌ "Error: Os { code: 13, kind: PermissionDenied, message: \"Permission denied\" }"

**Logging:**
- Use `tracing` crate for structured logging
- Default to minimal output; verbose logging should be opt-in via `--verbose` flag
- Log levels:
  - `error!`: Critical failures that prevent core functionality
  - `warn!`: Recoverable issues (e.g., failed DNS lookup, missing optional data)
  - `info!`: High-level operations (e.g., "Scanning connections for PID 1234")
  - `debug!`: Detailed internal state (e.g., "Parsed 42 TCP connections from /proc/net/tcp")
  - `trace!`: Very detailed debugging information
- Never log sensitive data (full command lines, connection payloads) at INFO level
- Use structured logging with fields when appropriate

**Example:**
```rust
use tracing::{debug, info, warn};

pub fn scan_connections(pid: u32) -> Result<Vec<Connection>> {
    info!(pid = pid, "Scanning connections");
    
    let tcp_conns = parse_proc_net_tcp(pid).map_err(|e| {
        warn!(pid = pid, error = %e, "Failed to parse /proc/net/tcp");
        e
    })?;
    
    debug!(pid = pid, count = tcp_conns.len(), "Found TCP connections");
    Ok(tcp_conns)
}
```

## Testing & Demo Mode

**Unit testing:**
- Write unit tests for parsing logic using fixture files, not live system state
- Store test fixtures in `tests/fixtures/` directory
- Test edge cases: empty files, malformed data, unusual connection states
- Place unit tests in the same file using `#[cfg(test)]` module
- Use `#[test]` attribute for test functions

**Demo mode:**
- Implement `--demo` flag that uses synthetic data from `src/demo/fixtures.rs`
- Demo data should be realistic and showcase all features:
  - Mix of ESTABLISHED, LISTEN, TIME_WAIT connections
  - Both local and remote endpoints
  - Some "suspicious" patterns for detection rule testing
- Demo mode should work without any system access (no /proc reads, no root)
- Use demo mode for screenshots, documentation, and hackathon presentations

**Test coverage expectations:**
- All parsing functions should have tests
- Graph building logic should have tests with known inputs/outputs
- TUI rendering doesn't need pixel-perfect tests, but state management should be tested
- When Kiro generates code, include basic tests for non-trivial logic
- Use `cargo test` to run all tests
- Use `cargo tarpaulin` or similar for coverage reports

**Example test:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_proc_net_tcp_line() {
        let test_cases = vec![
            TestCase {
                name: "localhost listen",
                line: "  0: 0100007F:1F90 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000",
                pid: 1234,
                want_ip: "127.0.0.1",
                want_port: 8080,
                want_state: ConnectionState::Listen,
            },
        ];
        
        for tc in test_cases {
            let conn = parse_tcp_line(tc.line, tc.pid).expect(&tc.name);
            assert_eq!(conn.local_addr.ip().to_string(), tc.want_ip, "{}", tc.name);
            assert_eq!(conn.local_addr.port(), tc.want_port, "{}", tc.name);
            assert_eq!(conn.state, tc.want_state, "{}", tc.name);
        }
    }
}
```

## TUI Design Guidelines

**Layout principles:**
- **Must work in 80x24:** This is the minimum supported terminal size
- **Scale gracefully:** Larger terminals should show more detail, not just bigger text
- **Prioritize readability:** Information density is good, but not at the cost of clarity
- **Predictable layout:** Center map, side panels, bottom status bar should stay consistent

**Visual hierarchy:**
- Most important info (target process, active connections) should be immediately visible
- Use colors and icons to highlight states, not to decorate
- Avoid visual clutter: every element should serve a purpose
- Provide clear focus indicators (which node is selected, which panel is active)

**Color usage:**
- Use colors semantically: green = healthy, yellow = warning, red = problem, gray = inactive
- Ensure sufficient contrast for readability in various terminal color schemes
- Support both light and dark terminal backgrounds (test both)
- Provide a way to disable colors if needed (respect `NO_COLOR` env var)

**Icon usage:**
- Icons should enhance, not replace, text labels
- Always provide ASCII fallback for terminals without Unicode support
- Test that icons render correctly in common terminal emulators

**Interaction:**
- Keyboard shortcuts should be intuitive and discoverable
- Show available shortcuts in a help panel or status bar
- Provide immediate visual feedback for user actions (selection changes, refresh, etc.)
- Handle terminal resize gracefully (redraw layout, don't crash)

## Halloween Theme Rules

**Theme philosophy:**
- Halloween icons (coffin ⚰, pumpkin 🎃, ghost 👻, skull 💀) add personality but must never reduce legibility
- Themes are cosmetic; they don't change functionality or data
- Both default and Halloween themes should be equally usable for serious work

**Implementation requirements:**
- Define a `Theme` trait with methods like `node_icon()`, `state_color()`
- Implement `DefaultTheme` and `HalloweenTheme` as separate structs implementing the trait
- Make theme switching easy: `--theme=halloween` flag or runtime toggle with `t` key
- Store theme choice in app state, not scattered across rendering code

**Halloween theme specifics:**
- Target process: Coffin ⚰ or tombstone 🪦 (center node)
- ESTABLISHED connections: Pumpkin 🎃 (healthy, active)
- LISTEN sockets: Pumpkin 🎃 or jack-o'-lantern (welcoming)
- TIME_WAIT, FIN_WAIT: Ghost 👻 (fading away)
- CLOSE_WAIT, failed: Skull 💀 or ☠ (dead/dying)
- Color palette: Dark purples, oranges, greens; avoid pure black/white

**Readability checks:**
- Test both themes in 80x24 and larger terminals
- Verify that connection states are distinguishable at a glance
- Ensure text remains readable over any background colors
- Get feedback from actual users (SREs, security engineers) on usability

**Example theme trait:**
```rust
use ratatui::style::Color;

pub trait Theme {
    fn name(&self) -> &'static str;
    fn process_icon(&self) -> &'static str;
    fn endpoint_icon(&self, suspicious: bool) -> &'static str;
    fn state_icon(&self, state: ConnectionState) -> &'static str;
    fn state_color(&self, state: ConnectionState) -> Color;
}

pub struct HalloweenTheme;

impl Theme for HalloweenTheme {
    fn name(&self) -> &'static str {
        "halloween"
    }

    fn process_icon(&self) -> &'static str {
        "⚰"
    }

    fn endpoint_icon(&self, suspicious: bool) -> &'static str {
        if suspicious { "💀" } else { "🎃" }
    }

    // ... etc
}
```

## Summary

When generating or refactoring ntomb code, Kiro should:
- Keep modules focused and separated by concern
- Use clear, descriptive names and avoid clever tricks
- Handle errors gracefully with informative messages using Result and anyhow
- Write testable code with demo mode support
- Prioritize TUI readability and usability
- Implement themes as swappable, cosmetic layers using traits
- Follow Rust idioms and best practices (The Rust Book, Rust API Guidelines)
- Use `rustfmt` for consistent formatting
- Use `clippy` for linting and catching common mistakes
- Leverage Rust's type system for safety and expressiveness

This style guide ensures ntomb remains maintainable, extensible, and pleasant to work with for both developers and end users.
