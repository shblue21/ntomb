# Requirements Document

## Introduction

This feature adds a "process focus mode" to ntomb's Graveyard (network topology map) that allows users to select a specific process and visualize only that process's network connections. The default mode shows the entire host's network map, while process focus mode displays the selected process as the central node with its remote endpoints radiating outward.

## Glossary

- **Connection**: A data structure representing a TCP network connection (includes local/remote addresses, ports, and state)
- **Process Mapping**: The process of identifying which process (PID, name) owns a network connection (socket)
- **Graveyard**: ntomb's network topology map widget (uses Canvas + Braille markers)
- **Host Mode**: The default view mode showing all connections for the entire host
- **Process Mode**: A focused view mode that filters and displays only connections for a specific process
- **inode**: A unique number identifying a socket in Linux
- **procfs**: Linux's /proc virtual filesystem

## Requirements

### Requirement 1

**User Story:** As a user, I want Connection data to include process ownership information, so that I can identify which process owns each network connection.

#### Acceptance Criteria

1. WHEN a Connection is created THEN the ntomb system SHALL include optional pid field (Option<i32>) and optional process_name field (Option<String>)
2. WHEN collect_connections() is called THEN the ntomb system SHALL initialize pid and process_name fields to None
3. WHEN Connection data is displayed THEN the ntomb system SHALL show process information if available, or indicate "unknown" if not available

### Requirement 2

**User Story:** As a user running ntomb on Linux, I want the system to automatically map connections to their owning processes, so that I can see which process is responsible for each connection.

#### Acceptance Criteria

1. WHEN attach_process_info() is called on Linux THEN the ntomb system SHALL read /proc/<pid>/fd/* to find socket inodes
2. WHEN a socket inode is found THEN the ntomb system SHALL map the inode to the corresponding pid
3. WHEN a pid is identified THEN the ntomb system SHALL read /proc/<pid>/comm to obtain the process name
4. WHEN process mapping completes THEN the ntomb system SHALL update Connection.pid and Connection.process_name fields for matched connections
5. IF permission errors occur during /proc reading THEN the ntomb system SHALL continue operation without the affected process information and log a warning
6. IF /proc filesystem is unavailable THEN the ntomb system SHALL return Ok(()) without modifying connections

### Requirement 3

**User Story:** As a user running ntomb on non-Linux systems (e.g., macOS), I want the application to work gracefully without process mapping, so that I can still use the network visualization features.

#### Acceptance Criteria

1. WHEN attach_process_info() is called on non-Linux systems THEN the ntomb system SHALL return Ok(()) without performing any operations
2. WHEN process information is unavailable THEN the ntomb system SHALL display connections without process details
3. WHEN process focus mode is attempted without process data THEN the ntomb system SHALL display an informative message

### Requirement 4

**User Story:** As a user, I want to toggle between Host mode and Process mode in the Graveyard view, so that I can focus on a specific process's network activity.

#### Acceptance Criteria

1. WHEN ntomb starts THEN the Graveyard SHALL display in Host mode by default
2. WHEN the user presses 'p' key with a connection selected THEN the ntomb system SHALL switch to Process mode showing only that process's connections
3. WHEN in Process mode and user presses 'p' key THEN the ntomb system SHALL return to Host mode
4. WHEN switching modes THEN the ntomb system SHALL update the Graveyard display immediately

### Requirement 5

**User Story:** As a user in Process mode, I want the Graveyard to show the selected process as the central node with its connections radiating outward, so that I can understand that process's network topology.

#### Acceptance Criteria

1. WHEN in Process mode THEN the Graveyard central node SHALL display "PROC: <process_name> (<pid>)" instead of "HOST"
2. WHEN in Process mode THEN the Graveyard SHALL display only connections belonging to the selected process
3. WHEN the selected process has no active connections THEN the Graveyard SHALL display "(no active connections for this process)" message
4. WHEN in Process mode THEN the connection lines and endpoint nodes SHALL follow the same color coding rules as Host mode

### Requirement 6

**User Story:** As a user, I want to see process information in the Active Connections list, so that I can identify which process owns each connection at a glance.

#### Acceptance Criteria

1. WHEN displaying a connection in the Active Connections list THEN the ntomb system SHALL append "[<process_name>(<pid>)]" if process information is available
2. WHEN process information is unavailable for a connection THEN the ntomb system SHALL omit the process tag from that connection's display

### Requirement 7

**User Story:** As a user, I want the status bar to show the current Graveyard mode and available key bindings, so that I can understand the current state and available actions.

#### Acceptance Criteria

1. WHEN in Host mode THEN the status bar SHALL include "P:Focus Process" hint
2. WHEN in Process mode THEN the status bar SHALL include "P:Back to Host" hint
3. WHEN mode changes THEN the status bar SHALL update to reflect the new mode immediately
