# Design Document: Code Refactoring

## Overview

This design document describes the structural refactoring of the ntomb codebase. The goal is to split the monolithic `ui.rs` (2,090 lines) and `app.rs` (938 lines) into smaller, focused modules while preserving all existing functionality.

## Architecture

### Current Structure

```
src/
├── main.rs          (119 lines) - Entry point + event loop
├── app.rs           (938 lines) - AppState + configs + business logic
├── ui.rs            (2,090 lines) - All UI rendering + theme + helpers
├── net/mod.rs       (305 lines) - Network data collection
└── procfs/mod.rs    (202 lines) - Linux process mapping
```

### Target Structure

```
src/
├── main.rs              (~50 lines) - Entry point only
├── app/
│   ├── mod.rs           (~300 lines) - AppState
│   ├── config.rs        (~150 lines) - RefreshConfig, LatencyConfig, GraveyardSettings
│   └── event.rs         (~100 lines) - Keyboard event handling
├── net/
│   └── mod.rs           (unchanged) - Network data collection
├── procfs/
│   └── mod.rs           (unchanged) - Linux process mapping
├── ui/
│   ├── mod.rs           (~100 lines) - draw() + re-exports
│   ├── banner.rs        (~100 lines) - render_banner
│   ├── graveyard.rs     (~600 lines) - render_network_map + helpers
│   ├── inspector.rs     (~150 lines) - render_soul_inspector
│   ├── grimoire.rs      (~150 lines) - render_grimoire
│   └── status_bar.rs    (~100 lines) - render_status_bar
└── theme/
    ├── mod.rs           (~100 lines) - Color constants + Theme trait
    ├── default.rs       (~50 lines) - Default theme implementation
    └── overdrive.rs     (~100 lines) - Kiroween Overdrive theme
```

## Components and Interfaces

### 1. Theme Module (`src/theme/`)

```rust
// theme/mod.rs
pub mod default;
pub mod overdrive;

// Color constants (moved from ui.rs)
pub const NEON_PURPLE: Color = Color::Rgb(187, 154, 247);
pub const PUMPKIN_ORANGE: Color = Color::Rgb(255, 158, 100);
pub const BLOOD_RED: Color = Color::Rgb(247, 118, 142);
pub const TOXIC_GREEN: Color = Color::Rgb(158, 206, 106);
pub const BONE_WHITE: Color = Color::Rgb(169, 177, 214);

// Re-export theme functions
pub use default::*;
pub use overdrive::*;
```

```rust
// theme/overdrive.rs
use crate::net::ConnectionState;
use crate::app::LatencyBucket;

pub fn get_overdrive_icon(state: ConnectionState, latency: LatencyBucket) -> &'static str;
pub fn get_overdrive_status_text(state: ConnectionState) -> &'static str;
pub fn get_stats_label(overdrive_enabled: bool) -> &'static str;
```

### 2. App Module (`src/app/`)

```rust
// app/mod.rs
pub mod config;
pub mod event;

pub use config::*;

pub struct AppState {
    // ... fields unchanged
}

impl AppState {
    pub fn new() -> Self;
    pub fn on_tick(&mut self);
    // ... other methods
}
```

```rust
// app/config.rs
pub struct GraveyardSettings { ... }
pub struct LatencyConfig { ... }
pub struct RefreshConfig { ... }
pub enum GraveyardMode { Host, Process }
pub enum LatencyBucket { Low, Medium, High, Unknown }
```

```rust
// app/event.rs
use crossterm::event::KeyCode;
use crate::app::AppState;

pub fn handle_key_event(app: &mut AppState, key: KeyCode) -> bool;
```

### 3. UI Module (`src/ui/`)

```rust
// ui/mod.rs
mod banner;
mod graveyard;
mod inspector;
mod grimoire;
mod status_bar;

pub use graveyard::{EndpointNode, EndpointType, classify_endpoint, ...};

pub fn draw(f: &mut Frame, app: &mut AppState) {
    // Layout setup
    render_banner(f, chunks[0], app);
    render_network_map(f, body_chunks[0], app);
    render_soul_inspector(f, right_chunks[0], app);
    render_grimoire(f, right_chunks[1], app);
    render_status_bar(f, chunks[2], app);
}
```

### 4. Main Entry Point (`src/main.rs`)

```rust
mod app;
mod net;
mod procfs;
mod theme;
mod ui;

fn main() -> Result<()> {
    // Terminal setup
    enable_raw_mode()?;
    // ...
    
    let res = app::run(&mut terminal);
    
    // Terminal teardown
    disable_raw_mode()?;
    // ...
}
```

## Data Models

No changes to data models. All existing structs remain unchanged:
- `Connection` (in `net/mod.rs`)
- `AppState` (moved to `app/mod.rs`)
- `GraveyardSettings`, `LatencyConfig`, `RefreshConfig` (moved to `app/config.rs`)

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Theme function consistency
*For any* connection state and latency bucket combination, calling theme functions with the same inputs SHALL always return the same output (icon, color, or text).
**Validates: Requirements 2.4, 2.5**

### Property 2: Keyboard handler state consistency
*For any* valid key code and initial AppState, the event handler SHALL produce the same resulting state as the pre-refactoring implementation.
**Validates: Requirements 6.2**

### Property 3: Overdrive mode icon consistency
*For any* connection state and latency bucket, the overdrive icon function SHALL return the same icon as the pre-refactoring implementation.
**Validates: Requirements 6.4**

## Error Handling

No changes to error handling. The refactoring is purely structural and does not modify error handling behavior.

## Testing Strategy

### Unit Testing

- **Theme tests**: Verify color constants and theme functions return expected values
- **Config tests**: Verify default values and configuration behavior
- **Event tests**: Verify keyboard handlers produce correct state changes

### Property-Based Testing

The following property-based tests will be implemented using the `proptest` crate:

1. **Theme consistency property**: Generate random connection states and verify theme functions are deterministic
2. **Event handler property**: Generate random key sequences and verify state transitions match expected behavior
3. **Overdrive icon property**: Generate random state/latency combinations and verify icon consistency

### Test Migration

All 46 existing tests will be migrated to their respective modules:
- UI-related tests → `ui/graveyard.rs`, `ui/mod.rs`
- Theme-related tests → `theme/overdrive.rs`
- App state tests → `app/mod.rs`
- Config tests → `app/config.rs`

### Regression Testing

After refactoring:
1. Run `cargo test` to verify all 46 tests pass
2. Run `cargo build` to verify no compilation errors
3. Run `cargo clippy` to verify no new warnings
4. Manual visual inspection to verify UI renders identically
