# Implementation Plan: UI Skeleton

## Phase 1: Core Layout Structure

- [x] 1. Set up main layout system
  - [x] 1.1 Create src/ui.rs module with draw() function
    - Implement 3-tier vertical layout (Banner 8줄, Body 가변, Status Bar 3줄)
    - Use Ratatui Layout with Constraint::Length and Constraint::Min
    - _Requirements: 1.1_
  - [x] 1.2 Implement body horizontal split
    - Left 65% for Graveyard (network map)
    - Right 35% for detail panels
    - _Requirements: 1.2_
  - [x] 1.3 Implement right panel vertical split
    - Top 60% for Soul Inspector
    - Bottom 40% for Grimoire
    - _Requirements: 1.3_

- [x] 2. Implement Banner rendering
  - [x] 2.1 Create render_banner() function
    - ASCII art logo "NTOMB"
    - Title "The Necromancer's Terminal v0.9.0"
    - Tagline "Revealing the unseen connections of the undead."
    - _Requirements: 2.1, 2.2, 2.3_
  - [x] 2.2 Add global stats display
    - Total Souls count
    - BPF Radar status indicator
    - _Requirements: 2.4_
  - [x] 2.3 Apply banner styling
    - Double border type
    - Neon Purple border color
    - Gradient text effect (Purple → Orange)
    - _Requirements: 2.5_

- [x] 3. Checkpoint - Verify layout foundation
  - Ensure all tests pass, ask the user if questions arise.

## Phase 2: Graveyard (Network Map) Implementation

- [x] 4. Set up Canvas widget for Graveyard
  - [x] 4.1 Create render_network_map() function
    - Use Canvas widget with Braille marker
    - Set x_bounds and y_bounds to [0.0, 100.0]
    - Apply Rounded border with Neon Purple color
    - _Requirements: 3.1_
  - [x] 4.2 Implement summary line
    - Display Endpoints, Listening, Total counts
    - Position above canvas area
    - _Requirements: 3.6_

- [x] 5. Implement HOST node rendering
  - [x] 5.1 Draw central HOST node
    - Position at HOST_CENTER (50.0, 50.0)
    - Display "⚰️ HOST" label in Host mode
    - Display "⚰️ PROC: name (pid)" in Process mode
    - Use Pumpkin Orange color with Bold modifier
    - _Requirements: 3.2_
  - [x] 5.2 Handle empty state
    - Display "The graveyard is quiet..." in Host mode
    - Display "(no active connections for this process)" in Process mode
    - Use Bone White color with Italic modifier
    - _Requirements: 3.5_

- [x] 6. Implement endpoint aggregation
  - [x] 6.1 Group connections by remote IP
    - Create HashMap<String, Vec<&Connection>>
    - Exclude LISTEN state and "0.0.0.0" addresses
    - _Requirements: 3.3_
  - [x] 6.2 Sort and limit endpoints
    - Sort by connection count (descending)
    - Limit to MAX_NODES (12)
    - Display "+N more" indicator when exceeded
    - _Requirements: 3.4_

- [x] 7. Implement endpoint node rendering
  - [x] 7.1 Create EndpointNode struct
    - Fields: label, x, y, state, conn_count, latency_bucket
    - _Requirements: 4.1-4.6_
  - [x] 7.2 Implement state-based icon mapping
    - ESTABLISHED → 🎃
    - TIME_WAIT → 👻
    - CLOSE_WAIT → 💀
    - SYN_SENT → ⏳
    - LISTEN → 👂
    - Other → 🌐
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_
  - [x] 7.3 Implement state-based color mapping
    - ESTABLISHED → Toxic Green
    - TIME_WAIT/CLOSE_WAIT → Pumpkin Orange
    - CLOSE → Blood Red
    - Other → Bone White
    - _Requirements: 4.1, 4.2, 4.3, 4.6_
  - [x] 7.4 Implement label truncation
    - Max 15 characters
    - Truncate to 12 chars + "..."
    - _Requirements: 4.7_

- [x] 8. Implement connection edge rendering
  - [x] 8.1 Draw edges from HOST to endpoints
    - Use CanvasLine from HOST_CENTER to endpoint position
    - _Requirements: 5.1_
  - [x] 8.2 Apply state-based edge colors
    - ESTABLISHED → Toxic Green
    - TIME_WAIT/CLOSE_WAIT → Pumpkin Orange
    - SYN_SENT/SYN_RECV → Yellow
    - CLOSE → Blood Red
    - Other → pulse_phase animated color
    - _Requirements: 5.2, 5.3, 5.4, 5.5, 5.6_

- [x] 9. Checkpoint - Verify Graveyard rendering
  - Ensure all tests pass, ask the user if questions arise.

## Phase 3: Soul Inspector Implementation

- [x] 10. Implement Soul Inspector panel
  - [x] 10.1 Create render_soul_inspector() function
    - Split into 3 sections: info, sparkline, socket list
    - Apply Rounded border with Neon Purple color
    - _Requirements: 6.6_
  - [x] 10.2 Display target information
    - TARGET with coffin icon
    - PID, PPID, USER fields
    - STATE with color-coded status
    - _Requirements: 6.1_
  - [x] 10.3 Display refresh interval
    - Show current refresh_ms value
    - Apply color coding based on value
    - Highlight if recently changed (Bold, Underline)
    - _Requirements: 6.2, 6.3_
  - [x] 10.4 Implement Traffic History Sparkline
    - Use Sparkline widget
    - Display last 60 samples
    - Use Toxic Green color
    - _Requirements: 6.4_
  - [x] 10.5 Display Open Sockets List
    - Show sample socket entries
    - Format: protocol://address:port (state)
    - _Requirements: 6.5_

- [x] 11. Checkpoint - Verify Soul Inspector
  - Ensure all tests pass, ask the user if questions arise.

## Phase 4: Grimoire (Connection List) Implementation

- [x] 12. Implement Grimoire panel
  - [x] 12.1 Create render_grimoire() function
    - Use List widget with ListItem
    - Apply Rounded border with Pumpkin Orange color
    - _Requirements: 7.7_
  - [x] 12.2 Format connection entries
    - Active: "local:port → remote:port [STATE]"
    - Listen: "local:port [LISTEN]"
    - Add index number prefix
    - _Requirements: 7.1, 7.2, 7.3_
  - [x] 12.3 Add process info tags
    - Append "[name(pid)]" if pid and process_name exist
    - Use Cyan color for tag
    - _Requirements: 7.4_
  - [x] 12.4 Implement selection highlighting
    - Check selected_connection index
    - Apply Deep Indigo background color
    - Use render_stateful_widget with ListState
    - _Requirements: 7.5_
  - [x] 12.5 Apply state-based colors
    - ESTABLISHED → Toxic Green
    - LISTEN → Bone White
    - TIME_WAIT/CLOSE_WAIT → Pumpkin Orange
    - CLOSE → Blood Red
    - _Requirements: 7.6_

- [x] 13. Checkpoint - Verify Grimoire
  - Ensure all tests pass, ask the user if questions arise.

## Phase 5: Status Bar Implementation

- [x] 14. Implement Status Bar
  - [x] 14.1 Create render_status_bar() function
    - Apply Double border with Neon Purple color
    - Left-align content
    - _Requirements: 8.8_
  - [x] 14.2 Display keyboard hints
    - Priority 1: Q:R.I.P, ↑↓:Navigate, P:Focus/Back
    - Priority 2: +/-:Speed, A:Anim, H:Theme, t:Labels
    - Priority 3: TAB:Switch Pane, F1:Help
    - _Requirements: 8.1_
  - [x] 14.3 Implement mode-specific hints
    - Host mode: "P:Focus Process"
    - Process mode: "P:Back to Host"
    - _Requirements: 8.2, 8.3_
  - [x] 14.4 Create build_toggle_indicators() function
    - Format: [A:ON/OFF] [H:ON/OFF] [t:ON/OFF]
    - ON → Toxic Green, OFF → Bone White
    - _Requirements: 8.4, 8.5, 8.6_
  - [x] 14.5 Implement width-based hint truncation
    - Calculate available width
    - Add hints by priority until space exhausted
    - _Requirements: 8.7_

- [x] 15. Checkpoint - Verify Status Bar
  - Ensure all tests pass, ask the user if questions arise.

## Phase 6: Keyboard Interaction

- [x] 16. Implement key handlers in main.rs
  - [x] 16.1 Exit handlers
    - 'q', 'Q', Esc → app.running = false
    - _Requirements: 9.1_
  - [x] 16.2 Navigation handlers
    - ↑ → select_previous_connection()
    - ↓ → select_next_connection()
    - _Requirements: 9.2_
  - [x] 16.3 Mode toggle handler
    - 'p', 'P' → toggle_graveyard_mode()
    - _Requirements: 9.3_
  - [x] 16.4 Panel switch handler
    - Tab → switch_panel() (placeholder)
    - _Requirements: 9.4_
  - [x] 16.5 Refresh rate handlers
    - '+', '=' → increase_refresh_rate()
    - '-', '_' → decrease_refresh_rate()
    - _Requirements: 9.5, 9.6_
  - [x] 16.6 Visual toggle handlers
    - 'a', 'A' → toggle animations_enabled
    - 'h', 'H' → toggle overdrive_enabled
    - 't', 'T' → toggle labels_enabled
    - _Requirements: 9.7, 9.8, 9.9_

- [x] 17. Checkpoint - Verify keyboard interaction
  - Ensure all tests pass, ask the user if questions arise.

## Phase 7: Animation System

- [x] 18. Implement animation state in AppState
  - [x] 18.1 Add animation fields
    - pulse_phase: f32 (0.0 ~ 1.0)
    - zombie_blink: bool
    - last_tick: Instant
    - last_blink: Instant
    - _Requirements: 10.1, 10.3_
  - [x] 18.2 Implement on_tick() method
    - Update pulse_phase every 100ms (+0.05, wrap at 1.0)
    - Toggle zombie_blink every 500ms
    - Update traffic_history with animated data
    - _Requirements: 10.1, 10.3, 10.4_

- [x] 19. Implement color interpolation
  - [x] 19.1 Create interpolate_color() function
    - Input: two RGB tuples, ratio (0.0-1.0)
    - Output: interpolated Color::Rgb
    - _Requirements: 10.2_
  - [x] 19.2 Apply pulse animation to edges
    - Use pulse_phase for "Other" state edges
    - Interpolate between purple shades
    - _Requirements: 10.2_

- [x] 20. Implement graceful degradation
  - [x] 20.1 Check animations_enabled flag
    - If disabled, use static colors
    - Maintain same information display
    - _Requirements: 10.5_

- [x] 21. Checkpoint - Verify animation system
  - Ensure all tests pass, ask the user if questions arise.

## Phase 8: Color Palette Compliance

- [x] 22. Define color constants
  - [x] 22.1 Add color constants to src/ui.rs
    - NEON_PURPLE: Color::Rgb(187, 154, 247)
    - PUMPKIN_ORANGE: Color::Rgb(255, 158, 100)
    - BLOOD_RED: Color::Rgb(247, 118, 142)
    - TOXIC_GREEN: Color::Rgb(158, 206, 106)
    - BONE_WHITE: Color::Rgb(169, 177, 214)
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_

- [x] 23. Apply consistent color usage
  - [x] 23.1 Verify border colors
    - Banner, Soul Inspector, Status Bar → Neon Purple
    - Grimoire → Pumpkin Orange
    - _Requirements: 11.1, 11.2_
  - [x] 23.2 Verify state colors
    - Healthy/Active → Toxic Green
    - Warning → Pumpkin Orange
    - Error → Blood Red
    - Inactive → Bone White
    - _Requirements: 11.3, 11.4, 11.5_
  - [x] 23.3 Verify selection background
    - Selected items → Deep Indigo (#2f334d)
    - _Requirements: 11.6_

- [x] 24. Final Checkpoint - Complete UI verification
  - Ensure all tests pass, ask the user if questions arise.

## Remaining Tasks (Future Improvements)

- [ ] 25. Terminal resize handling
  - [ ] 25.1 Implement resize event handler
    - Detect terminal size changes
    - Recalculate layout proportions
    - _Requirements: 1.4_
  - [ ] 25.2 Test minimum size (80x24)
    - Verify all components visible
    - Ensure no layout overflow
    - _Requirements: 1.5_

- [ ] 26. Panel focus system
  - [ ] 26.1 Implement panel focus state
    - Track currently focused panel
    - Visual indicator for focused panel
    - _Requirements: 9.4_
  - [ ] 26.2 Implement Tab key cycling
    - Cycle: Graveyard → Soul Inspector → Grimoire
    - Update focus indicator on switch
    - _Requirements: 9.4_

- [ ] 27. Help panel overlay
  - [ ] 27.1 Create help panel widget
    - List all keyboard shortcuts
    - Show current toggle states
  - [ ] 27.2 Implement F1/? toggle
    - Show/hide help overlay
    - Overlay on top of main UI

- [ ]* 28. Unit tests for UI functions
  - [ ]* 28.1 Test classify_latency()
    - Boundary conditions (49ms, 50ms, 200ms, 201ms)
    - None handling
    - _Requirements: 1.2, 1.3, 1.4_
  - [ ]* 28.2 Test calculate_endpoint_position()
    - Position within bounds
    - Even distribution around ring
  - [ ]* 28.3 Test interpolate_color()
    - Ratio 0.0, 0.5, 1.0
    - Boundary clamping
  - [ ]* 28.4 Test build_toggle_indicators()
    - ON/OFF state colors
    - Format correctness

