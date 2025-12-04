# Requirements Document

## Introduction

This feature adds configurable refresh intervals for both UI rendering and data collection in ntomb. Users can adjust the refresh rates to balance between responsiveness and system resource usage, and change these settings at runtime using keyboard shortcuts.

## Glossary

- **UI Refresh Interval**: The time between consecutive screen redraws (tick rate)
- **Data Collection Interval**: The time between consecutive network connection scans
- **Tick**: A single iteration of the main event loop that updates animations and UI state
- **Connection Refresh**: The process of reading /proc/net/tcp and updating the connection list
- **Runtime Configuration**: Settings that can be changed while the application is running

## Requirements

### Requirement 1

**User Story:** As a user, I want to adjust the UI refresh rate, so that I can balance between smooth animations and CPU usage.

#### Acceptance Criteria

1. WHEN the application starts THEN the ntomb system SHALL use a default UI refresh interval of 100ms
2. WHEN the user presses '+' or '=' key THEN the ntomb system SHALL decrease the UI refresh interval by 50ms (faster refresh)
3. WHEN the user presses '-' or '_' key THEN the ntomb system SHALL increase the UI refresh interval by 50ms (slower refresh)
4. WHEN the UI refresh interval reaches 50ms THEN the ntomb system SHALL prevent further decreases
5. WHEN the UI refresh interval reaches 1000ms THEN the ntomb system SHALL prevent further increases

### Requirement 2

**User Story:** As a user, I want to adjust the data collection rate, so that I can control how frequently network connections are scanned.

#### Acceptance Criteria

1. WHEN the application starts THEN the ntomb system SHALL use a default data collection interval of 1000ms
2. WHEN the user presses Shift+'+' or Shift+'=' key THEN the ntomb system SHALL decrease the data collection interval by 500ms (faster collection)
3. WHEN the user presses Shift+'-' or Shift+'_' key THEN the ntomb system SHALL increase the data collection interval by 500ms (slower collection)
4. WHEN the data collection interval reaches 500ms THEN the ntomb system SHALL prevent further decreases
5. WHEN the data collection interval reaches 5000ms THEN the ntomb system SHALL prevent further increases

### Requirement 3

**User Story:** As a user, I want to see the current refresh rates on screen, so that I know what settings are active.

#### Acceptance Criteria

1. WHEN the UI is rendered THEN the ntomb system SHALL display the current UI refresh interval in milliseconds
2. WHEN the UI is rendered THEN the ntomb system SHALL display the current data collection interval in milliseconds
3. WHEN a refresh interval is changed THEN the ntomb system SHALL update the displayed value immediately
4. WHEN displaying refresh intervals THEN the ntomb system SHALL show them in the Soul Inspector panel

### Requirement 4

**User Story:** As a user, I want visual feedback when I change refresh rates, so that I know my input was registered.

#### Acceptance Criteria

1. WHEN a refresh interval is changed THEN the ntomb system SHALL briefly highlight the new value
2. WHEN a refresh interval reaches its minimum or maximum limit THEN the ntomb system SHALL provide visual indication
3. WHEN displaying refresh intervals THEN the ntomb system SHALL use color coding to indicate performance impact (green for normal, yellow for high frequency, red for very high frequency)

### Requirement 5

**User Story:** As a user, I want the status bar to show the keyboard shortcuts for changing refresh rates, so that I can discover this feature easily.

#### Acceptance Criteria

1. WHEN the status bar is rendered THEN the ntomb system SHALL include hints for UI refresh rate controls
2. WHEN the status bar is rendered THEN the ntomb system SHALL include hints for data collection rate controls
3. WHEN the status bar text is too long for the terminal width THEN the ntomb system SHALL prioritize essential shortcuts

### Requirement 6

**User Story:** As a developer, I want the refresh intervals to be properly synchronized, so that the application remains responsive and doesn't waste resources.

#### Acceptance Criteria

1. WHEN the UI refresh interval is changed THEN the ntomb system SHALL apply the new interval to the event polling timeout
2. WHEN the data collection interval is changed THEN the ntomb system SHALL update the next scheduled collection time
3. WHEN data collection takes longer than the UI refresh interval THEN the ntomb system SHALL not block UI updates
4. WHEN the application is idle THEN the ntomb system SHALL not consume excessive CPU resources
