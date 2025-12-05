# Implementation Plan

- [ ] 1. Add LayoutConfig struct and constants
  - [ ] 1.1 Define LayoutConfig struct with ring radii fields and is_adaptive flag
    - Add struct to `src/ui/graveyard.rs`
    - Include fields: ring_low, ring_medium, ring_high, edge_padding, is_adaptive
    - _Requirements: 1.1, 2.1_
  - [ ] 1.2 Add adaptive layout constants
    - Define ADAPTIVE_THRESHOLD, RING_RATIO_LOW/MEDIUM/HIGH, EDGE_PADDING_PERCENT, MIN_EDGE_PADDING
    - _Requirements: 1.3, 1.4_

- [ ] 2. Implement calculate_layout_config function
  - [ ] 2.1 Create calculate_layout_config function
    - Accept canvas_width and canvas_height parameters
    - Calculate available radius from smaller dimension
    - Return LayoutConfig with scaled or fixed radii based on threshold
    - _Requirements: 1.1, 1.2, 1.3, 3.1_
  - [ ]* 2.2 Write property test for adaptive scaling
    - **Property 1: Adaptive Scaling**
    - **Validates: Requirements 1.1, 1.2, 3.1**
  - [ ]* 2.3 Write property test for fixed fallback
    - **Property 2: Fixed Fallback**
    - **Validates: Requirements 1.3**

- [ ] 3. Update calculate_endpoint_position function
  - [ ] 3.1 Modify calculate_endpoint_position to accept LayoutConfig
    - Change function signature to include layout: &LayoutConfig parameter
    - Replace hardcoded RING_RADII with layout.ring_low/medium/high
    - _Requirements: 1.2, 2.1, 2.3_
  - [ ]* 3.2 Write property test for bounds invariant
    - **Property 3: Bounds Invariant**
    - **Validates: Requirements 1.4, 3.2**
  - [ ]* 3.3 Write property test for ring ratio preservation
    - **Property 4: Ring Ratio Preservation**
    - **Validates: Requirements 2.1, 2.3**
  - [ ]* 3.4 Write property test for angular separation
    - **Property 5: Angular Separation**
    - **Validates: Requirements 2.2**

- [ ] 4. Integrate adaptive layout into render_network_map
  - [ ] 4.1 Update render_network_map to use adaptive layout
    - Calculate canvas dimensions from Rect
    - Call calculate_layout_config with dimensions
    - Pass LayoutConfig to calculate_endpoint_position calls
    - Update draw_latency_rings to use adaptive radii
    - _Requirements: 1.1, 1.2, 2.1, 3.1_

- [ ] 5. Update draw_latency_rings function
  - [ ] 5.1 Modify draw_latency_rings to accept LayoutConfig
    - Change function signature to include layout: &LayoutConfig parameter
    - Use layout ring radii instead of hardcoded RING_RADII constant
    - _Requirements: 1.1, 2.1_

- [ ] 6. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ]* 7. Write unit tests for edge cases
  - [ ]* 7.1 Add unit tests for calculate_layout_config
    - Test zero/negative dimensions fallback
    - Test boundary at ADAPTIVE_THRESHOLD
    - Test extreme aspect ratios
    - _Requirements: 1.3, 3.1_

