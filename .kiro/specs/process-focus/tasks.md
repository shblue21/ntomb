# Implementation Plan

- [x] 1. Extend Connection struct
  - [x] 1.1 Add pid, process_name fields to Connection
    - Add `pub pid: Option<i32>`, `pub process_name: Option<String>` fields to Connection struct in src/net/mod.rs
    - Initialize new fields to None in collect_connections()
    - _Requirements: 1.1, 1.2_
  - [ ]* 1.2 Property test: Connection initialization consistency
    - **Property 1: Connection initialization consistency**
    - **Validates: Requirements 1.1, 1.2**

- [x] 2. Implement procfs module (Linux process mapping)
  - [x] 2.1 Create src/procfs/mod.rs module
    - Create new module file and add mod declaration to src/main.rs
    - _Requirements: 2.1_
  - [x] 2.2 Implement attach_process_info() function
    - Linux: Iterate /proc/<pid>/fd/* to extract socket inodes
    - Linux: Read process name from /proc/<pid>/comm
    - Non-Linux: Implement no-op that returns Ok(())
    - Graceful handling of permission errors (continue, log only)
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 3.1_
  - [ ]* 2.3 Property test: Process mapping integrity
    - **Property 2: Process mapping integrity**
    - **Validates: Requirements 2.2, 2.4**
  - [ ]* 2.4 Unit test: Non-Linux graceful handling
    - Verify attach_process_info() returns Ok(()) on non-Linux
    - _Requirements: 3.1, 3.2_

- [x] 3. Add Graveyard mode to AppState
  - [x] 3.1 Define GraveyardMode enum
    - Add GraveyardMode enum to src/app.rs (Host, Process)
    - Implement Default trait with Host as default
    - _Requirements: 4.1_
  - [x] 3.2 Extend AppState fields
    - Add graveyard_mode: GraveyardMode field
    - Add selected_process_pid: Option<i32> field
    - Add selected_connection: Option<usize> field
    - Set initial values in AppState::new()
    - _Requirements: 4.1_
  - [x] 3.3 Implement mode switching methods
    - Implement focus_process_of_selected_connection()
    - Implement clear_process_focus()
    - Implement toggle_graveyard_mode()
    - _Requirements: 4.2, 4.3_
  - [x] 3.4 Property test: Mode toggle consistency
    - **Property 3: Mode toggle consistency**
    - **Validates: Requirements 4.2, 4.3**

- [x] 4. Integrate procfs with AppState
  - [x] 4.1 Add process mapping to refresh_connections()
    - Call procfs::attach_process_info() after collect_connections()
    - Only runs on Linux, continues on failure
    - _Requirements: 2.4, 3.1_

- [x] 5. Checkpoint - Verify data layer
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Add key bindings
  - [x] 6.1 Add 'p' key handler to main.rs
    - Add KeyCode::Char('p') matching
    - Call app.toggle_graveyard_mode()
    - _Requirements: 4.2, 4.3_
  - [x] 6.2 Add connection selection key bindings
    - Modify Up/Down key handlers to change selected_connection
    - Add range checking logic (0 to connections.len() - 1)
    - _Requirements: 4.2_

- [x] 7. Modify Graveyard rendering (Process mode support)
  - [x] 7.1 Add connection filtering logic to render_network_map()
    - Determine connections to display based on GraveyardMode
    - Process mode: Filter by selected_process_pid
    - Host mode: Use all connections
    - _Requirements: 5.2_
  - [ ]* 7.2 Property test: Process filtering accuracy
    - **Property 4: Process filtering accuracy**
    - **Validates: Requirements 5.2**
  - [x] 7.3 Change center node label in render_network_map()
    - Host mode: Display "⚰️ HOST"
    - Process mode: Display "⚰️ PROC: <name> (<pid>)"
    - _Requirements: 5.1_
  - [ ]* 7.4 Property test: Center node label accuracy
    - **Property 6: Center node label accuracy**
    - **Validates: Requirements 5.1**
  - [x] 7.5 Handle empty connections in render_network_map()
    - Display "(no active connections for this process)" message when Process mode has no connections
    - _Requirements: 5.3_

- [x] 8. Update Active Connections list in render_grimoire()
  - [x] 8.1 Add process info to connection display
    - Add "[name(pid)]" tag if pid, process_name exist
    - Omit tag if not present
    - _Requirements: 6.1, 6.2_
  - [ ]* 8.2 Property test: Connection display formatting
    - **Property 5: Connection display formatting**
    - **Validates: Requirements 6.1, 6.2**
  - [x] 8.3 Highlight selected connection
    - Emphasize item at selected_connection index using Style with bg color or modifier
    - _Requirements: 4.2_

- [x] 9. Update status bar in render_status_bar()
  - [x] 9.1 Add mode-specific hint text
    - Host mode: Display "P:Focus Process"
    - Process mode: Display "P:Back to Host"
    - Pass app state to render_status_bar() to access graveyard_mode
    - _Requirements: 7.1, 7.2, 7.3_

- [x] 10. Final Checkpoint - Verify complete functionality
  - Ensure all tests pass, ask the user if questions arise.
