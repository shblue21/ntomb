# Requirements Document

## Introduction

This document defines requirements for structural refactoring of the ntomb codebase. Currently, `ui.rs` is 2,090 lines which is too large, and `app.rs` has multiple responsibilities mixed together. This refactoring will modularize the code to improve maintainability, testability, and extensibility.

## Glossary

- **AppState**: The struct that manages the entire application state
- **Theme**: A visual theme system that defines colors, icons, and styles
- **Graveyard**: The main canvas area that visualizes network topology
- **Soul Inspector**: A panel that displays detailed information about selected connections
- **Grimoire**: A panel that displays the list of active connections
- **Overdrive Mode**: A visual mode with enhanced Halloween theming

## Requirements

### Requirement 1: UI Module Separation

**User Story:** As a developer, I want the UI code to be split into separate modules by component, so that I can easily find and modify specific UI elements.

#### Acceptance Criteria

1. WHEN the ui module is loaded THEN the System SHALL expose a single `draw()` function from `ui/mod.rs`
2. WHEN rendering the banner THEN the System SHALL use code from `ui/banner.rs`
3. WHEN rendering the network map THEN the System SHALL use code from `ui/graveyard.rs`
4. WHEN rendering the soul inspector THEN the System SHALL use code from `ui/inspector.rs`
5. WHEN rendering the grimoire THEN the System SHALL use code from `ui/grimoire.rs`
6. WHEN rendering the status bar THEN the System SHALL use code from `ui/status_bar.rs`
7. WHEN all UI modules are combined THEN the System SHALL maintain identical visual output to the current implementation

### Requirement 2: Theme System Separation

**User Story:** As a developer, I want theme-related code separated into its own module, so that I can easily add new themes or modify existing ones.

#### Acceptance Criteria

1. WHEN the theme module is loaded THEN the System SHALL provide color constants from `theme/mod.rs`
2. WHEN using default theme THEN the System SHALL load styles from `theme/default.rs`
3. WHEN using overdrive theme THEN the System SHALL load styles from `theme/overdrive.rs`
4. WHEN switching themes THEN the System SHALL apply the new theme without restart
5. WHEN a theme function is called THEN the System SHALL return consistent colors and icons based on connection state

### Requirement 3: App Module Separation

**User Story:** As a developer, I want application state and configuration separated into logical modules, so that the codebase is easier to understand and test.

#### Acceptance Criteria

1. WHEN the app module is loaded THEN the System SHALL expose `AppState` from `app/mod.rs`
2. WHEN configuration is needed THEN the System SHALL load settings from `app/config.rs`
3. WHEN handling keyboard events THEN the System SHALL use handlers from `app/event.rs`
4. WHEN the main loop runs THEN the System SHALL delegate event handling to the event module

### Requirement 4: Main Entry Point Simplification

**User Story:** As a developer, I want the main.rs file to be minimal and focused only on bootstrapping, so that the application entry point is clear and simple.

#### Acceptance Criteria

1. WHEN the application starts THEN the main.rs SHALL only handle terminal setup and teardown
2. WHEN the main loop runs THEN the main.rs SHALL delegate to app module functions
3. WHEN main.rs is complete THEN the file SHALL contain fewer than 60 lines of code

### Requirement 5: Test Separation

**User Story:** As a developer, I want tests to be co-located with their respective modules, so that I can easily find and run relevant tests.

#### Acceptance Criteria

1. WHEN UI tests exist THEN the System SHALL place them in their respective UI submodule files
2. WHEN theme tests exist THEN the System SHALL place them in theme module files
3. WHEN app tests exist THEN the System SHALL place them in app module files
4. WHEN all tests are run THEN the System SHALL pass all 46 existing tests

### Requirement 6: Preserve Existing Functionality

**User Story:** As a user, I want all existing features to work exactly as before after refactoring, so that the refactoring does not break any functionality.

#### Acceptance Criteria

1. WHEN the application runs THEN the System SHALL display identical UI to the pre-refactoring version
2. WHEN keyboard shortcuts are pressed THEN the System SHALL respond identically to the pre-refactoring version
3. WHEN animations are enabled THEN the System SHALL render particles and effects identically
4. WHEN overdrive mode is toggled THEN the System SHALL apply themed icons and colors identically
5. WHEN connections are refreshed THEN the System SHALL display process information identically
