# Implementation Plan: Graveyard VFX & Kiroween Enhancement

## Phase 1: State and Toggle Foundation

- [x] 1. Add GraveyardSettings to AppState
  - [x] 1.1 Create GraveyardSettings struct in src/app.rs
    - Add `animations_enabled: bool` (default: true)
    - Add `labels_enabled: bool` (default: true)
    - Add `overdrive_enabled: bool` (default: false)
    - Implement `Default` trait
    - _Requirements: 5.1, 5.2, 5.3_
  - [x] 1.2 Create LatencyConfig struct in src/app.rs
    - Add `low_threshold_ms: u64` (default: 50)
    - Add `high_threshold_ms: u64` (default: 200)
    - Create `LatencyBucket` enum (Low, Medium, High, Unknown)
    - _Requirements: 1.2, 1.3, 1.4_
  - [x] 1.3 Add fields to AppState
    - Add `pub graveyard_settings: GraveyardSettings`
    - Add `pub latency_config: LatencyConfig`
    - Initialize in `AppState::new()`
    - _Requirements: 5.7_

- [x] 2. Implement keyboard toggle handlers
  - [x] 2.1 Add 'A' key handler in src/main.rs
    - Toggle `app.graveyard_settings.animations_enabled`
    - Handle both 'a' and 'A' (case-insensitive)
    - _Requirements: 2.4, 5.1_
  - [x] 2.2 Add 'H' key handler in src/main.rs
    - Toggle `app.graveyard_settings.overdrive_enabled`
    - Handle both 'h' and 'H' (case-insensitive)
    - _Requirements: 4.1, 5.2_
  - [x] 2.3 Add 't' key handler in src/main.rs
    - Toggle `app.graveyard_settings.labels_enabled`
    - Handle both 't' and 'T' (case-insensitive)
    - _Requirements: 3.6, 5.3_

- [x] 3. Update status bar with toggle indicators
  - [x] 3.1 Add toggle status display in render_status_bar()
    - Show `[A:ON/OFF]` for animations
    - Show `[H:ON/OFF]` for overdrive
    - Show `[t:ON/OFF]` for labels
    - Use appropriate colors (Toxic Green for ON, Bone White for OFF)
    - _Requirements: 5.6_
  - [x] 3.2 Add keyboard hints for new toggles
    - Add "A:Anim" hint
    - Add "H:Theme" hint
    - Add "t:Labels" hint
    - _Requirements: 5.6_

- [x] 4. Checkpoint - Verify toggle foundation
  - Ensure all tests pass, ask the user if questions arise.

## Phase 2: Latency Rings and Position Calculation

- [x] 5. Implement latency bucket classification
  - [x] 5.1 Create classify_latency() function in src/ui.rs
    - Input: latency_ms (Option<u64>), config (LatencyConfig)
    - Return: LatencyBucket enum
    - Handle None case → LatencyBucket::Unknown
    - _Requirements: 1.2, 1.3, 1.4, 1.5_
  - [x] 5.2 Add latency data to endpoint aggregation
    - Extend EndpointNode struct with `latency_bucket: LatencyBucket`
    - Calculate bucket during endpoint collection
    - _Requirements: 1.1_

- [x] 6. Draw latency rings on canvas
  - [x] 6.1 Create draw_latency_rings() function in src/ui.rs
    - Define RING_RADII constant: [15.0, 25.0, 35.0]
    - Define HOST_CENTER constant: (50.0, 50.0)
    - Draw 3 concentric dotted circles using Braille markers
    - Use Bone White color with decreasing opacity
    - _Requirements: 1.1, 1.6_
  - [x] 6.2 Conditionally render rings
    - Only draw rings if at least one endpoint has latency data
    - Skip rings if all endpoints are LatencyBucket::Unknown
    - _Requirements: 1.5_

- [x] 7. Update endpoint position calculation
  - [x] 7.1 Modify calculate_endpoint_position() in src/ui.rs
    - Accept LatencyBucket parameter
    - Map bucket to ring radius
    - Distribute endpoints evenly around their assigned ring
    - Add small jitter to prevent overlap
    - _Requirements: 1.2, 1.3, 1.4_
  - [x] 7.2 Implement fallback for unknown latency
    - Use middle ring (radius 25) as default
    - Maintain existing radial layout behavior
    - _Requirements: 1.5_

- [x] 8. Checkpoint - Verify latency rings
  - Ensure all tests pass, ask the user if questions arise.

## Phase 3: Edge Particle Animation

- [x] 9. Implement particle position calculation
  - [x] 9.1 Create particle_position() function in src/ui.rs
    - Input: start (f64, f64), end (f64, f64), pulse_phase (f32), offset (f32)
    - Calculate parametric position t = (pulse_phase + offset) % 1.0
    - Return interpolated (x, y) coordinates
    - _Requirements: 2.2_
  - [x] 9.2 Define particle constants
    - PARTICLE_COUNT: 3 per edge
    - PARTICLE_OFFSETS: [0.0, 0.33, 0.66]
    - PARTICLE_SYMBOL: "●"
    - _Requirements: 2.1_

- [x] 10. Integrate particles into edge rendering
  - [x] 10.1 Modify edge drawing in render_network_map()
    - Check `app.graveyard_settings.animations_enabled`
    - If enabled: draw base line + particles
    - If disabled: draw base line only
    - _Requirements: 2.4, 2.5_
  - [x] 10.2 Apply particle colors based on edge state
    - Normal connection: Neon Purple particles
    - Healthy/active: Toxic Green particles
    - Warning/high-latency: Pumpkin Orange particles
    - _Requirements: 2.3_

- [x] 11. Ensure graceful degradation
  - [x] 11.1 Verify static rendering without animations
    - Connection lines remain visible
    - State colors still applied to edges
    - No visual information lost
    - _Requirements: 2.5, 2.6, 5.4_

- [x] 12. Checkpoint - Verify particle animation
  - Ensure all tests pass, ask the user if questions arise.

## Phase 4: Endpoint Type Icons and Colors

- [x] 13. Implement endpoint classification
  - [x] 13.1 Create EndpointType enum in src/ui.rs
    - Variants: Localhost, Private, Public, ListenOnly
    - _Requirements: 3.1, 3.2, 3.3, 3.5_
  - [x] 13.2 Create classify_endpoint() function
    - Check for localhost (127.0.0.1, ::1, 0.0.0.0)
    - Check for RFC1918 private ranges
    - Check for LISTEN-only sockets (remote = 0.0.0.0:0)
    - Default to Public for all other IPs
    - _Requirements: 3.1, 3.2, 3.3, 3.5_
  - [x] 13.3 Implement heavy talker detection
    - Create is_heavy_talker() function
    - Input: connection count, all endpoint counts
    - Return true if in top 5 by count
    - _Requirements: 3.4_

- [x] 14. Apply icons and colors to endpoints
  - [x] 14.1 Create icon() method on EndpointType
    - Localhost → "⚰️"
    - Private → "🪦"
    - Public → "🎃"
    - ListenOnly → "🕯"
    - _Requirements: 3.1, 3.2, 3.3, 3.5_
  - [x] 14.2 Create color() method on EndpointType
    - Localhost → TOXIC_GREEN
    - Private → BONE_WHITE
    - Public → PUMPKIN_ORANGE
    - ListenOnly → NEON_PURPLE
    - _Requirements: 3.1, 3.2, 3.3, 3.5_
  - [x] 14.3 Add heavy talker badge
    - Append "👑" to icon if is_heavy_talker() returns true
    - _Requirements: 3.4_

- [x] 15. Integrate with endpoint rendering
  - [x] 15.1 Update EndpointNode struct
    - Add `endpoint_type: EndpointType`
    - Add `is_heavy_talker: bool`
    - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - [x] 15.2 Apply classification during aggregation
    - Call classify_endpoint() for each unique remote IP
    - Calculate heavy talker status after all endpoints collected
    - _Requirements: 3.4_
  - [x] 15.3 Render with type-specific visuals
    - Use endpoint_type.icon() for node symbol
    - Use endpoint_type.color() for node color
    - Add 👑 badge for heavy talkers
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [x] 16. Implement label toggle
  - [x] 16.1 Conditionally render endpoint labels
    - Check `app.graveyard_settings.labels_enabled`
    - If enabled: show IP:port text near node
    - If disabled: show icon only
    - _Requirements: 3.6_

- [x] 17. Checkpoint - Verify endpoint icons
  - Ensure all tests pass, ask the user if questions arise.

## Phase 5: Kiroween Overdrive Mode

- [x] 18. Implement overdrive visual transformations
  - [x] 18.1 Create get_overdrive_icon() function
    - ESTABLISHED → "🟢👻"
    - High latency → "🔥🎃"
    - CLOSE_WAIT/TIME_WAIT → "💀"
    - _Requirements: 4.2, 4.3, 4.4_
  - [x] 18.2 Create get_overdrive_status_text() function
    - "Alive" → "Haunting"
    - "Listening" → "Summoning"
    - "Closing" → "Fading"
    - _Requirements: 4.5_
  - [x] 18.3 Create get_stats_label() function
    - Normal: "Connections"
    - Overdrive: "Spirits"
    - _Requirements: 4.5_

- [x] 19. Integrate overdrive into rendering
  - [x] 19.1 Update endpoint icon selection
    - Check `app.graveyard_settings.overdrive_enabled`
    - If enabled: use overdrive icons
    - If disabled: use normal icons
    - _Requirements: 4.2, 4.3, 4.4_
  - [x] 19.2 Update Soul Inspector text
    - Apply overdrive status text when enabled
    - _Requirements: 4.5_
  - [x] 19.3 Update header stats
    - Change "Total Souls" label based on overdrive
    - _Requirements: 4.5_

- [x] 20. Verify tone compliance
  - [x] 20.1 Review all overdrive text
    - Ensure calm, informative tone per security-domain.md
    - No fear-mongering language
    - No absolute claims about security status
    - _Requirements: 4.6_

- [x] 21. Checkpoint - Verify overdrive mode
  - Ensure all tests pass, ask the user if questions arise.

## Phase 6: Performance and Testing

- [x] 22. Performance optimization
  - [x] 22.1 Implement endpoint limit
    - Define MAX_VISIBLE_ENDPOINTS constant (30)
    - Show top N endpoints by connection count
    - Display "... and N more" indicator
    - _Requirements: 6.3, 6.4_
  - [x] 22.2 Optimize particle rendering
    - Skip particles for edges outside visible area
    - Reduce particle count if > 50 edges
    - _Requirements: 6.1, 6.5_
  - [x] 22.3 Add animation complexity auto-reduction
    - Monitor frame time
    - If consistently > 100ms, reduce particle count
    - _Requirements: 6.5_

- [x] 23. Unit tests
  - [x]* 23.1 Test endpoint classification
    - Test RFC1918 detection (10.x, 172.16-31.x, 192.168.x)
    - Test localhost detection
    - Test public IP fallback
    - _Requirements: 3.1, 3.2, 3.5_
  - [x]* 23.2 Test latency bucket classification
    - Test low threshold (< 50ms)
    - Test medium range (50-200ms)
    - Test high threshold (> 200ms)
    - Test unknown (None) handling
    - _Requirements: 1.2, 1.3, 1.4, 1.5_
  - [x]* 23.3 Test heavy talker detection
    - Test with various connection counts
    - Test edge cases (< 5 endpoints)
    - _Requirements: 3.4_
  - [x]* 23.4 Test particle position calculation
    - Test at phase 0.0, 0.5, 1.0
    - Test with various offsets
    - _Requirements: 2.2_

- [x] 24. Integration testing
  - [x]* 24.1 Test toggle persistence
    - Verify toggles maintain state across frames
    - Verify toggles apply immediately
    - _Requirements: 5.7_
  - [x]* 24.2 Test mode combinations
    - Host mode + Overdrive
    - Process mode + Animations off
    - All toggles off
    - _Requirements: 5.4_

- [x] 25. Manual testing checklist
  - [x] 25.1 Visual inspection
    - Verify ring layout looks correct
    - Verify particle animation is smooth
    - Verify icons render correctly
    - _Requirements: 1.1, 2.1, 3.1_
  - [x] 25.2 Performance testing
    - Test with 50 connections
    - Test with 100 connections
    - Test with 200+ connections
    - Verify FPS targets (10 FPS animated, 30 FPS static)
    - _Requirements: 6.1, 6.2_
  - [x] 25.3 Accessibility testing
    - Verify readability with animations off
    - Verify readability with labels off
    - Test in SSH session
    - _Requirements: 5.4, 5.5_
  - [x] 25.4 Cross-platform testing
    - Test on Linux
    - Test on macOS
    - _Requirements: 6.1, 6.2_

- [x] 26. Final Checkpoint - Complete verification
  - Ensure all tests pass, ask the user if questions arise.
